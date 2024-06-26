use bevy::prelude::*;
use bevy_trait_query::RegisterExt;
use izhikevich::IzhikevichNeuron;
use leaky::LifNeuron;
use uom::si::f64::Time;

use crate::synapses::synapse::update_synapses;

pub struct NeuronRuntimePlugin;

fn update_clock(mut clock: ResMut<Clock>) {
    clock.time += clock.tau;
}

impl Plugin for NeuronRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Clock {
            time: 0.0,
            tau: 0.025,
        })
        .add_event::<SpikeEvent>()
        .add_systems(Update, update_synapses)
        .add_systems(PostUpdate, update_clock);
    }
}

#[derive(Event, Debug)]
pub struct SpikeEvent {
    pub time: Time,
    pub neuron: Entity,
}

#[derive(Debug)]
pub struct Spike {
    pub time: Time,
    pub neuron: Entity,
}

#[derive(Component, Debug)]
pub struct SpikeRecorder {
    pub spikes: Vec<Spike>,
}
