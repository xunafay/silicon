use bevy::app::{App, Plugin};
use bevy_trait_query::RegisterExt;
use izhikevich::IzhikevichNeuron;
use leaky::LifNeuron;

pub mod izhikevich;
pub mod leaky;

#[bevy_trait_query::queryable]
pub trait Neuron {
    fn update(&mut self, tau: f64) -> bool;
    fn get_membrane_potential(&self) -> f64;
    fn add_membrane_potential(&mut self, delta_v: f64) -> f64;
}

#[bevy_trait_query::queryable]
pub trait NeuronVisualizer {
    fn activation_percent(&self) -> f64;
}

pub struct NeuronPlugin;

impl Plugin for NeuronPlugin {
    fn build(&self, app: &mut App) {
        app.register_component_as::<dyn Neuron, LifNeuron>()
            .register_component_as::<dyn Neuron, IzhikevichNeuron>()
            .register_component_as::<dyn NeuronVisualizer, LifNeuron>()
            .register_component_as::<dyn NeuronVisualizer, IzhikevichNeuron>();
    }
}
