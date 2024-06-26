use bevy::prelude::*;
use bevy_trait_query::RegisterExt;
use izhikevich::IzhikevichNeuron;
use leaky::LifNeuron;
use uom::si::f64::Time;

use crate::synapses::synapse::update_synapses;

pub mod izhikevich;
pub mod leaky;

pub struct NeuronRuntimePlugin;

#[bevy_trait_query::queryable]
pub trait Neuron {
    fn update(&mut self, tau: Time) -> bool;
    fn get_membrane_potential(&self) -> f64;
    fn add_membrane_potential(&mut self, delta_v: f64) -> f64;
}

#[bevy_trait_query::queryable]
pub trait NeuronVisualizer {
    fn activation_percent(&self) -> f64;
}

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
        .add_systems(Update, update_synapses)
        .register_component_as::<dyn Neuron, LifNeuron>()
        .register_component_as::<dyn Neuron, IzhikevichNeuron>()
        .register_component_as::<dyn NeuronVisualizer, LifNeuron>()
        .register_component_as::<dyn NeuronVisualizer, IzhikevichNeuron>()
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
