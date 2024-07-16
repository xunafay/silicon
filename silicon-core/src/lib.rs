#![warn(missing_docs)]
#![forbid(unsafe_code)]

//! Silicon core is a library for building spiking neural networks in bevy.

use bevy::{prelude::Resource, reflect::Reflect};

#[bevy_trait_query::queryable]
/// Core trait for neurons. Simulator queries for this trait and calls update for every simulation time tick.
pub trait Neuron {
    /// Update the neuron for the given time step.
    fn update(&mut self, tau: f64) -> bool;
    /// Get the membrane potential of the neuron.
    fn get_membrane_potential(&self) -> f64;
    /// Add to the membrane potential of the neuron, subtract by providing a negative value.
    fn insert_current(&mut self, delta_v: f64) -> f64;
}

/// Allows a neuron to be visualized in 3D.
#[bevy_trait_query::queryable]
pub trait NeuronVisualizer {
    /// Get the percentage of activation of the neuron.
    fn activation_percent(&self) -> f64;
}

/// This trait allows for implementations like STDP, where the synapse needs to know when a neuron spiked.
/// Your neuron implementation should call this method when it spikes.
/// We recommend clearing the spikes after reading them.
/// This is not enforced by the trait. But prevents memory from getting out of hand during long simulation times.
#[bevy_trait_query::queryable]
pub trait SpikeRecorder {
    /// Record a spike at the given time.
    fn record_spike(&mut self, time: f64);
    /// Get the spikes that have been recorded.
    fn get_spikes(&self) -> Vec<f64>;
}

/// Clock is a high level resource that tracks the simulation time.
#[derive(Resource, Reflect)]
pub struct Clock {
    /// The total time that has been simulated in seconds.
    pub time: f64,
    /// The remaining time to simulate in seconds.
    pub time_to_simulate: f64,
    /// If true, the simulation will run indefinitely.
    pub run_indefinitely: bool,
    /// The time step of the simulation.
    pub tau: f64,
}
