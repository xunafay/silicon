use bevy::prelude::*;
use uom::si::f64::{ElectricPotential, ElectricalResistance, Time};

#[derive(Component, Debug)]
pub struct Neuron {
    pub membrane_potential: ElectricPotential,
    pub reset_potential: ElectricPotential,
    pub threshold_potential: ElectricPotential,
    pub resistance: ElectricalResistance,
}

#[derive(Component, Debug)]
pub struct LeakyNeuron {
    pub resting_potential: ElectricPotential,
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
