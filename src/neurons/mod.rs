use bevy::prelude::*;
use leaky::LifNeuron;
use synapse::update_synapses;
use uom::si::{f64::Time, time::second};

pub mod cortical_column;
pub mod leaky;
pub mod oscillating;
pub mod synapse;

pub struct NeuronRuntimePlugin;

pub trait Neuron {
    fn update(&mut self, tau: Time) -> bool;
    fn get_membrane_potential(&self) -> f64;
    fn add_membrane_potential(&mut self, delta_v: f64) -> f64;
}

pub trait NeuronVisualizer {
    fn activation_percent(&self) -> f64;
}

#[derive(Component, Debug)]
pub struct IzhikevichNeuron {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub v: f64,
    pub u: f64,
}

impl Neuron for IzhikevichNeuron {
    fn update(&mut self, tau: Time) -> bool {
        let v =
            self.v + tau.get::<second>() * (0.04 * self.v * self.v + 5.0 * self.v + 140.0 - self.u);
        let u = self.u + tau.get::<second>() * self.a * (self.b * self.v - self.u);
        self.v = v;
        self.u = u;
        if self.v >= 30.0 {
            self.v = self.c;
            self.u = self.u + self.d;
            return true;
        }

        false
    }

    fn get_membrane_potential(&self) -> f64 {
        self.v
    }

    fn add_membrane_potential(&mut self, delta_v: f64) -> f64 {
        self.v += delta_v;
        self.v
    }
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
        .add_systems(
            Update,
            (
                update_synapses::<IzhikevichNeuron>,
                update_synapses::<LifNeuron>,
            ),
        )
        .add_systems(PostUpdate, update_clock);
    }
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
