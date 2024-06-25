use super::{Neuron, SpikeEvent};
use bevy::prelude::*;

/// A component that allows a neuron to receive synapses.
#[derive(Component, Debug)]
pub struct AllowSynapse;

pub fn update_synapses<T: Component + Neuron>(
    mut synapse_query: Query<&Synapse>,
    mut spike_reader: EventReader<SpikeEvent>,
    mut neuron_query: Query<(Entity, &mut T)>,
) {
    // return;
    for spike_event in spike_reader.read() {
        for synapse in synapse_query.iter_mut() {
            if synapse.source == spike_event.neuron {
                let (_, mut target_neuron) = neuron_query.get_mut(synapse.target).unwrap();

                // let threshold_potential = target_neuron.threshold_potential.get::<millivolt>();
                // let resting_potential = neuron.resting_potential.get::<millivolt>();

                let delta_v = synapse.weight;
                // trace!("Synapse fired: {:?}, delta_v: {:?}", synapse, delta_v);
                match synapse.synapse_type {
                    SynapseType::Excitatory => {
                        target_neuron.add_membrane_potential(delta_v);
                    }
                    SynapseType::Inhibitory => {
                        target_neuron.add_membrane_potential(-delta_v);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub enum SynapseType {
    #[default]
    Excitatory,
    Inhibitory,
}

#[derive(Component, Debug)]
pub struct Synapse {
    pub weight: f64,
    pub delay: u32,
    pub source: Entity,
    pub target: Entity,
    pub synapse_type: SynapseType,
}
