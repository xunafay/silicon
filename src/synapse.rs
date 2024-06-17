use bevy::prelude::{Component, Entity};

#[derive(Debug)]
pub enum SynapseType {
    Excitatory,
    Inhibitory,
}

#[derive(Component, Debug)]
pub struct Synapse {
    pub weight: f64,
    pub delay: u32,
    pub source: Entity,
    pub target: Entity,
    pub synapse_type: SynapseType,
}

impl Synapse {
    pub fn new(
        source: Entity,
        target: Entity,
        weight: f64,
        delay: u32,
        synapse_type: SynapseType,
    ) -> Self {
        Synapse {
            weight,
            delay,
            source,
            target,
            synapse_type,
        }
    }
}
