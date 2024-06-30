#![allow(clippy::type_complexity)]

use analytics::MembranePlotter;
use bevy::{
    app::{App, Plugin, Update},
    hierarchy::DespawnRecursiveExt,
    prelude::{Commands, Component, Entity, Event, EventReader, EventWriter, Query, Res, ResMut},
    reflect::Reflect,
};
use bevy_trait_query::{One, RegisterExt};
use silicon_core::{Clock, Neuron, SpikeRecorder};
use synapses::{
    stdp::{StdpSettings, StdpSynapse},
    Synapse, SynapseType,
};
use time::update_clock;
use tracing::{info, trace, warn};
pub mod time;

#[derive(Event, Debug)]
pub struct SpikeEvent {
    pub time: f64,
    pub neuron: Entity,
}

#[derive(Debug)]
pub struct Spike {
    pub time: f64,
    pub neuron: Entity,
}

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Clock {
            time: 0.0,
            tau: 0.025,
            time_to_simulate: 0.0,
            run_indefinitely: false,
        })
        .insert_resource(StdpSettings {
            look_back: 1.0,
            update_interval: 1.0,
            next_update: -0.1,
        })
        .register_type::<Clock>()
        .register_type::<StdpSettings>()
        .register_type::<MembranePlotter>()
        .register_type::<SimpleSpikeRecorder>()
        .add_event::<SpikeEvent>()
        .register_component_as::<dyn SpikeRecorder, SimpleSpikeRecorder>()
        .add_systems(
            Update,
            (
                update_clock,
                update_neurons,
                update_synapses,
                update_stdp_synapses,
                prune_synapses,
            ),
        );
    }
}

#[allow(unused)]
fn exhaustive_zip<I, J>(
    mut iter1: I,
    mut iter2: J,
) -> impl Iterator<Item = (Option<I::Item>, Option<J::Item>)>
where
    I: Iterator,
    J: Iterator,
{
    std::iter::from_fn(move || {
        let next1 = iter1.next();
        let next2 = iter2.next();
        if next1.is_none() && next2.is_none() {
            None
        } else {
            Some((next1, next2))
        }
    })
}

fn average<T>(values: &[T]) -> Option<f64>
where
    T: Into<f64> + Clone,
{
    if values.is_empty() {
        return None;
    }

    Some(values.iter().map(|v| (v.clone()).into()).sum::<f64>() / values.len() as f64)
}

pub fn prune_synapses(
    mut synapse_query: Query<(Entity, One<&dyn Synapse>)>,
    mut commands: Commands,
) {
    for (entity, synapse) in synapse_query.iter_mut() {
        if synapse.get_weight() < 0.1 {
            info!("Pruning synapse {:?}", entity);
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// This needs to be rewritten to an event based system.
/// Too many bugs with the current implementation.
///
/// Updates the weights of STDP synapses.
/// Does not update the membrane potential of the connected neurons.
pub fn update_stdp_synapses(
    mut synapse_query: Query<(Entity, &mut StdpSynapse)>,
    mut neuron_query: Query<(Entity, One<&mut dyn SpikeRecorder>)>,
    mut stdp_settings: ResMut<StdpSettings>,
    clock: Res<Clock>,
) {
    stdp_settings.next_update -= clock.tau;
    if stdp_settings.next_update >= 0.0 {
        return;
    }

    stdp_settings.next_update = stdp_settings.update_interval;

    for (entity, mut synapse) in synapse_query.iter_mut() {
        if synapse.get_type() == SynapseType::Inhibitory {
            continue;
        }

        let pre_spikes: Vec<f64> = {
            let pre_neuron = neuron_query.get_mut(synapse.source);
            if pre_neuron.is_err() {
                warn!("No source neuron found for synapse: {:?}", synapse);
                continue;
            }

            let (_entity, mut pre_spike_recorder) = pre_neuron.unwrap();
            pre_spike_recorder
                .get_spikes()
                .iter()
                .filter(|s| **s > (clock.time - stdp_settings.look_back))
                .cloned()
                .collect()
        };

        let post_spikes: Vec<f64> = {
            let post_neuron = neuron_query.get_mut(synapse.target);
            if post_neuron.is_err() {
                warn!("No target neuron found for synapse: {:?}", synapse);
                continue;
            }

            let (_entity, mut post_spike_recorder) = post_neuron.unwrap();
            post_spike_recorder
                .get_spikes()
                .iter()
                .filter(|s| **s > (clock.time - stdp_settings.look_back))
                .cloned()
                .collect()
        };

        let pre = average(&pre_spikes);
        let post = average(&post_spikes);

        let delta_t = match (pre, post) {
            (Some(pre), Some(post)) => post - pre,
            (Some(_), None) => -clock.tau * 2.0, // If the post-synaptic neuron did not spike, we want to decrease the weight.
            (None, Some(_)) => continue,
            (None, None) => continue,
        };

        // invert delta_t for inhibitory synapses so that the weight decreases when the postsynaptic neuron spikes after the presynaptic neuron.
        let delta_t = match synapse.get_type() {
            SynapseType::Excitatory => delta_t,
            SynapseType::Inhibitory => -delta_t,
        };

        if delta_t > 0.0 {
            let delta_w =
                synapse.stdp_params.a_plus * (-delta_t / synapse.stdp_params.tau_plus).exp();
            synapse.weight += delta_w;
            trace!(
                "Increasing weight by {} for synapse {:?} with new weight {}",
                delta_w,
                entity,
                synapse.weight
            );
        } else {
            let delta_w =
                synapse.stdp_params.a_minus * (delta_t / synapse.stdp_params.tau_minus).exp();
            synapse.weight += delta_w;
            trace!(
                "Decreasing weight by {} for synapse {:?} with new weight {}",
                delta_w,
                entity,
                synapse.weight
            );
        }

        // Clamp the weight to the min and max values.
        synapse.weight = synapse
            .weight
            .max(synapse.stdp_params.w_min)
            .min(synapse.stdp_params.w_max);

        // for (pre, post) in exhaustive_zip(pre_spikes.into_iter(), post_spikes.into_iter()) {
        //     debug!("Pre spike: {:?}, Post spike: {:?}", pre, post);
        //     let delta_t = match (pre, post) {
        //         (Some(pre), Some(post)) => post - pre,
        //         (Some(_), None) => -clock.tau,
        //         (None, Some(_)) => -clock.tau,
        //         (None, None) => continue,
        //     };

        //     if delta_t > 0.0 {
        //         let delta_w =
        //             synapse.stdp_params.a_plus * (-delta_t / synapse.stdp_params.tau_plus).exp();
        //         synapse.weight += delta_w;
        //         trace!(
        //             "Increasing weight by {} for synapse {:?} with new weight {}",
        //             delta_w,
        //             entity,
        //             synapse.weight
        //         );
        //     } else {
        //         let delta_w =
        //             synapse.stdp_params.a_minus * (delta_t / synapse.stdp_params.tau_minus).exp();
        //         synapse.weight += delta_w;
        //         trace!(
        //             "Decreasing weight by {} for synapse {:?} with new weight {}",
        //             delta_w,
        //             entity,
        //             synapse.weight
        //         );
        //     }

        //     // Clamp the weight to the min and max values.
        //     synapse.weight = synapse
        //         .weight
        //         .max(synapse.stdp_params.w_min)
        //         .min(synapse.stdp_params.w_max);
        // }
    }
}

pub fn update_synapses(
    synapse_query: Query<(Entity, One<&dyn Synapse>)>,
    mut spike_reader: EventReader<SpikeEvent>,
    mut neuron_query: Query<(Entity, One<&mut dyn Neuron>)>,
) {
    for spike_event in spike_reader.read() {
        for (_entity, synapse) in synapse_query.iter() {
            if synapse.get_presynaptic() == spike_event.neuron {
                let neuron = neuron_query.get_mut(synapse.get_postsynaptic());
                if neuron.is_err() {
                    // warn!("No target neuron found for synapse: {:?}", synapse);
                    continue;
                }

                let (_entity, mut target_neuron) = neuron.unwrap();

                match synapse.get_type() {
                    SynapseType::Excitatory => {
                        target_neuron.add_membrane_potential(synapse.get_weight());
                    }
                    SynapseType::Inhibitory => {
                        target_neuron.add_membrane_potential(-synapse.get_weight());
                    }
                }
            }
        }
    }
}

fn update_neurons(
    clock: ResMut<Clock>,
    mut neuron_query: Query<(
        Entity,
        One<&mut dyn Neuron>,
        Option<&mut MembranePlotter>,
        Option<One<&mut dyn SpikeRecorder>>,
    )>,
    mut spike_writer: EventWriter<SpikeEvent>,
) {
    if clock.time_to_simulate <= 0.0 {
        return;
    }

    for (entity, mut neuron, mut plotter, mut spike_recorder) in neuron_query.iter_mut() {
        let fired = neuron.update(clock.tau);
        if let Some(plotter) = &mut plotter {
            plotter.add_point(neuron.get_membrane_potential(), clock.time);
            if fired {
                plotter.add_spike(clock.time);
            }
        }

        if fired {
            spike_writer.send(SpikeEvent {
                time: clock.time,
                neuron: entity,
            });

            if let Some(spike_recorder) = &mut spike_recorder {
                trace!("Recording spike for neuron {:?} at {}", entity, clock.time);
                spike_recorder.record_spike(clock.time);
            }
        }
    }
}

#[derive(Debug, Component, Reflect)]
pub struct SimpleSpikeRecorder {
    max_spikes: usize,
    spikes: Vec<f64>,
}

impl SpikeRecorder for SimpleSpikeRecorder {
    fn record_spike(&mut self, time: f64) {
        self.spikes.push(time);
        if self.spikes.len() > self.max_spikes {
            self.spikes.remove(0);
        }
    }

    fn get_spikes(&mut self) -> Vec<f64> {
        self.spikes.clone()
    }
}

impl Default for SimpleSpikeRecorder {
    fn default() -> Self {
        SimpleSpikeRecorder {
            max_spikes: 1000,
            spikes: Vec::with_capacity(1000),
        }
    }
}
