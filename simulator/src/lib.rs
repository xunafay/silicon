#![allow(clippy::type_complexity)]

use analytics::MembranePlotter;
use bevy::{
    app::{App, Plugin, Update},
    hierarchy::DespawnRecursiveExt,
    prelude::{
        Commands, Component, Entity, Event, EventReader, EventWriter, Events, Query, Res, ResMut,
    },
    reflect::Reflect,
};
use bevy_trait_query::{One, RegisterExt};
use silicon_core::{Clock, Neuron, SpikeRecorder};
use synapses::{
    stdp::{StdpSettings, StdpSynapse},
    DeferredStdpEvent, Synapse, SynapseType,
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
                update_synapses_for_spikes,
                update_synapses,
                prune_synapses,
                // reward_modulated_stdp,
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

fn reward_modulated_stdp(
    mut deferred_stdp_events: ResMut<Events<DeferredStdpEvent>>,
    mut stdp_synapses: Query<(Entity, &mut StdpSynapse)>,
) {
    for event in deferred_stdp_events.drain() {
        let synapse = stdp_synapses
            .iter_mut()
            .find(|(entity, _)| *entity == event.synapse);

        if let Some((_, mut synapse)) = synapse {
            trace!(
                "applying stdp to {:?} with delta weight {} for a new weight of {}",
                event.synapse,
                event.delta_weight,
                synapse.weight + event.delta_weight
            );

            synapse.weight += event.delta_weight;
        }
    }
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

pub fn update_synapses(
    mut synapse_query: Query<(Entity, One<&mut dyn Synapse>)>,
    clock: Res<Clock>,
) {
    for (_, mut synapse) in &mut synapse_query {
        synapse.update(clock.tau);
    }
}

pub fn update_synapses_for_spikes(
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
    mut stdp_synapses: Query<(Entity, &mut StdpSynapse)>,
    mut spike_writer: EventWriter<SpikeEvent>,
    mut stdp_writer: EventWriter<DeferredStdpEvent>,
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
                // trace!("Recording spike for neuron {:?} at {}", entity, clock.time);
                spike_recorder.record_spike(clock.time);
            }

            stdp_synapses
                .iter_mut()
                .find(|(_, s)| s.get_presynaptic() == entity)
                .map(|(e, mut s)| {
                    // trace!("Registering pre-spike for synapse {:?}", entity);
                    let delta_w = s.register_pre_spike();
                    if let Some(delta_w) = delta_w {
                        stdp_writer.send(DeferredStdpEvent {
                            synapse: e,
                            delta_weight: delta_w,
                        });
                    }
                });

            stdp_synapses
                .iter_mut()
                .find(|(_, s)| s.get_postsynaptic() == entity)
                .map(|(e, mut s)| {
                    // trace!("Registering post-spike for synapse {:?}", entity);
                    let delta_w = s.register_post_spike();
                    if let Some(delta_w) = delta_w {
                        stdp_writer.send(DeferredStdpEvent {
                            synapse: e,
                            delta_weight: delta_w,
                        });
                    }
                });
        }
    }
}

#[derive(Debug, Component, Reflect)]
pub struct Classifier {
    pub neurons: Vec<Entity>,
    pub spikes: Vec<f64>,
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

    fn get_spikes(&self) -> Vec<f64> {
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
