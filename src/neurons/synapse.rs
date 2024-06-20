use bevy::prelude::*;
use uom::{
    si::{
        electric_potential::millivolt,
        f64::{ElectricPotential, Time as SiTime},
    },
    ConstZero,
};

use super::{leaky::LeakyNeuron, Neuron, Refactory, SpikeEvent};

pub fn update_synapses(
    mut synapse_query: Query<&Synapse>,
    mut spike_reader: EventReader<SpikeEvent>,
    mut neuron_query: Query<(Entity, &mut Neuron, &mut LeakyNeuron, &Refactory)>,
) {
    // return;
    for spike_event in spike_reader.read() {
        for synapse in synapse_query.iter_mut() {
            if synapse.source == spike_event.neuron {
                let (_, mut target_neuron, leaky, refactory) =
                    neuron_query.get_mut(synapse.target).unwrap();
                if refactory.refactory_counter > SiTime::ZERO {
                    continue;
                }

                let threshold_potential = target_neuron.threshold_potential.get::<millivolt>();
                let resting_potential = leaky.resting_potential.get::<millivolt>();

                let delta_v = synapse.weight * (threshold_potential - resting_potential);
                // trace!("Synapse fired: {:?}, delta_v: {:?}", synapse, delta_v);
                target_neuron.membrane_potential += ElectricPotential::new::<millivolt>(delta_v);
            }
        }
    }
}

#[derive(Debug)]
pub enum SynapseType {
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

impl Synapse {
    pub fn new(
        source: Entity,
        target: Entity,
        weight: f64,
        delay: u32,
        synapse_type: SynapseType,
    ) -> Self {
        Synapse {
            weight,
            delay,
            source,
            target,
            synapse_type,
        }
    }
}
