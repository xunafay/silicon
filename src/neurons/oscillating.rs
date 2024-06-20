use super::{
    leaky::LeakyNeuron, Clock, Neuron, OscillatingNeuron, Refactory, Spike, SpikeEvent,
    SpikeRecorder,
};
use bevy::prelude::*;
use uom::si::{
    electric_potential::millivolt,
    electrical_resistance::ohm,
    f64::{ElectricPotential, Time},
    time::second,
};

pub fn update_oscillating_neurons(
    clock: ResMut<Clock>,
    mut neuron_query: Query<(
        Entity,
        &mut Neuron,
        &mut LeakyNeuron,
        &mut OscillatingNeuron,
        Option<&mut SpikeRecorder>,
    )>,
    mut spike_writer: EventWriter<SpikeEvent>,
) {
    for (entity, mut neuron, _, oscillating, spike_recorder) in neuron_query.iter_mut() {
        let delta_v = (neuron.resistance.get::<ohm>()
            * (neuron.threshold_potential.get::<millivolt>() + 5.0
                - neuron.membrane_potential.get::<millivolt>()))
            * clock.tau
            * oscillating.frequency;

        neuron.membrane_potential += ElectricPotential::new::<millivolt>(delta_v);

        if neuron.membrane_potential >= neuron.threshold_potential {
            neuron.membrane_potential = neuron.reset_potential;

            if let Some(mut spike_recorder) = spike_recorder {
                spike_recorder.spikes.push(Spike {
                    time: Time::new::<second>(clock.time),
                    neuron: entity,
                });
            }

            spike_writer.send(SpikeEvent {
                time: Time::new::<second>(clock.time),
                neuron: entity,
            });
        }
    }
}
