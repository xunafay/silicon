use bevy::prelude::Component;

pub mod synapse;

/// A component that allows a neuron to receive synapses.
#[derive(Component, Debug)]
pub struct AllowSynapses;

#[derive(Debug, Copy, Clone, Default)]
pub enum SynapseType {
    #[default]
    Excitatory,
    Inhibitory,
}
