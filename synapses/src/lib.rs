use bevy::{
    app::{App, Plugin, Update},
    prelude::{Component, Entity, EventReader, Query},
};
use bevy_trait_query::One;
use neurons::Neuron;
use simple::SimpleSynapse;
use simulator::SpikeEvent;

pub mod simple;

/// A component that allows a neuron to receive synapses.
#[derive(Component, Debug)]
pub struct AllowSynapses;

#[derive(Debug, Copy, Clone, Default)]
pub enum SynapseType {
    #[default]
    Excitatory,
    Inhibitory,
}

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
                    // warn!("No target neuron found for synapse: {:?}", synapse);
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

pub struct SynapsePlugin;

impl Plugin for SynapsePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_synapses);
    }
}
