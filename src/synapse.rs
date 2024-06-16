use bevy::prelude::{Component, Entity};

#[derive(Component, Debug)]
pub struct Synapse {
    pub weight: f64,
    pub delay: u32,
    pub pre_synaptic_neuron: Entity,
    pub post_synaptic_neuron: Entity,
}

impl Synapse {
    pub fn new(pre_synaptic_neuron: Entity, post_synaptic_neuron: Entity) -> Self {
        Synapse {
            weight: 50.0,
            delay: 1,
            pre_synaptic_neuron,
            post_synaptic_neuron,
        }
    }
}
