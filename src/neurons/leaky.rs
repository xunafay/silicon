use bevy::prelude::*;
use uom::{
    si::{
        electric_potential::millivolt,
        f64::{ElectricPotential, Time as SiTime},
        time::second,
    },
    ConstZero,
};

use super::{Clock, Neuron, Refactory, Spike, SpikeEvent, SpikeRecorder};

#[derive(Component, Debug)]
pub struct LeakyNeuron {
    pub resting_potential: ElectricPotential,
}

pub fn update_leaky_neurons(
    clock: ResMut<Clock>,
    mut neuron_query: Query<(
        Entity,
        &mut Neuron,
        &mut LeakyNeuron,
        &mut Refactory,
        Option<&mut SpikeRecorder>,
    )>,
    mut spike_writer: EventWriter<SpikeEvent>,
) {
    for (entity, mut neuron, leaky, mut refactory, spike_recorder) in neuron_query.iter_mut() {
        if refactory.refactory_counter > SiTime::ZERO {
            refactory.refactory_counter -= SiTime::new::<second>(clock.tau);
            continue;
        }

        let delta_v = (leaky.resting_potential.get::<millivolt>()
            - neuron.membrane_potential.get::<millivolt>())
            * clock.tau;

        neuron.membrane_potential += ElectricPotential::new::<millivolt>(delta_v);

        if neuron.membrane_potential > neuron.threshold_potential {
            neuron.membrane_potential = neuron.reset_potential;
            refactory.refactory_counter = refactory.refractory_period;

            // trace!("Leaky neuron fired: {:?}", entity);
            spike_writer.send(SpikeEvent {
                time: SiTime::new::<second>(clock.time),
                neuron: entity,
            });

            if let Some(mut spike_recorder) = spike_recorder {
                spike_recorder.spikes.push(Spike {
                    time: SiTime::new::<second>(clock.time),
                    neuron: entity,
                });
            }
        }
    }
}
