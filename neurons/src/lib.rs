use bevy::app::{App, Plugin};
use bevy_trait_query::RegisterExt;
use izhikevich::IzhikevichNeuron;
use leaky::LifNeuron;
use silicon_core::{Neuron, NeuronVisualizer};

pub mod izhikevich;
pub mod leaky;

pub struct NeuronPlugin;

impl Plugin for NeuronPlugin {
    fn build(&self, app: &mut App) {
        app.register_component_as::<dyn Neuron, LifNeuron>()
            .register_component_as::<dyn Neuron, IzhikevichNeuron>()
            .register_component_as::<dyn NeuronVisualizer, LifNeuron>()
            .register_component_as::<dyn NeuronVisualizer, IzhikevichNeuron>();
    }
}
