use bevy::prelude::*;
use bevy_trait_query::One;

use crate::neurons::{Neuron, SpikeEvent};

use super::SynapseType;

pub fn update_synapses(
    synapse_query: Query<(Entity, &SimpleSynapse)>,
    mut spike_reader: EventReader<SpikeEvent>,
    mut neuron_query: Query<(Entity, One<&mut dyn Neuron>)>,
) {
    for spike_event in spike_reader.read() {
        for (_entity, synapse) in synapse_query.iter() {
            if synapse.source == spike_event.neuron {
                let neuron = neuron_query.get_mut(synapse.target);
                if neuron.is_err() {
                    warn!("No target neuron found for synapse: {:?}", synapse);
                    continue;
                }

                let (_entity, mut target_neuron) = neuron.unwrap();

                match synapse.synapse_type {
                    SynapseType::Excitatory => {
                        target_neuron.add_membrane_potential(synapse.weight);
                    }
                    SynapseType::Inhibitory => {
                        target_neuron.add_membrane_potential(-synapse.weight);
                    }
                }
            }
        }
    }
}

#[derive(Component, Debug)]
pub struct SimpleSynapse {
    pub weight: f64,
    pub delay: u32,
    pub source: Entity,
    pub target: Entity,
    pub synapse_type: SynapseType,
}
