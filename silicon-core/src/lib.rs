use bevy::{prelude::Resource, reflect::Reflect};

#[bevy_trait_query::queryable]
/// Core trait for neurons. Simulator queries for this trait and calls update for every simulation time tick.
pub trait Neuron {
    fn update(&mut self, tau: f64) -> bool;
    fn get_membrane_potential(&self) -> f64;
    fn add_membrane_potential(&mut self, delta_v: f64) -> f64;
}

#[bevy_trait_query::queryable]
pub trait NeuronVisualizer {
    fn activation_percent(&self) -> f64;
}

#[bevy_trait_query::queryable]
/// This trait allows for implementations like STDP, where the synapse needs to know when a neuron spiked.
/// Your neuron implementation should call this method when it spikes.
/// We recommend clearing the spikes after reading them.
/// This is not enforced by the trait. But prevents memory from getting out of hand during long simulation times.
pub trait SpikeRecorder {
    fn record_spike(&mut self, time: f64);
    fn get_spikes(&self) -> Vec<f64>;
}

#[derive(Resource, Reflect)]
pub struct Clock {
    pub time: f64,
    pub time_to_simulate: f64,
    pub run_indefinitely: bool,
    pub tau: f64,
}
