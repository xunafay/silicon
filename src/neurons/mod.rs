use bevy::prelude::*;
use leaky::update_leaky_neurons;
use oscillating::update_oscillating_neurons;
use synapse::update_synapses;
use uom::si::f64::{ElectricPotential, ElectricalResistance, Time};

pub mod cortical_column;
pub mod leaky;
pub mod oscillating;
pub mod synapse;

pub struct NeuronRuntimePlugin;

fn update_clock(mut clock: ResMut<Clock>) {
    clock.time += clock.tau;
}

#[derive(Resource)]
pub struct Clock {
    pub time: f64,
    pub tau: f64,
}

impl Plugin for NeuronRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Clock {
            time: 0.0,
            tau: 0.025,
        })
        .add_event::<SpikeEvent>()
        .add_systems(
            Update,
            (
                update_leaky_neurons,
                update_oscillating_neurons,
                update_synapses,
            ),
        )
        .add_systems(PostUpdate, update_clock);
    }
}

#[derive(Component, Debug)]
pub struct Neuron {
    pub membrane_potential: ElectricPotential,
    pub reset_potential: ElectricPotential,
    pub threshold_potential: ElectricPotential,
    pub resistance: ElectricalResistance,
}

#[derive(Component, Debug)]
pub struct Refactory {
    pub refractory_period: Time,
    pub refactory_counter: Time,
}

#[derive(Component, Debug)]
pub struct OscillatingNeuron {
    pub frequency: f64,
    pub amplitude: f64,
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
