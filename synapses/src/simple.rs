use bevy::prelude::{Component, Entity};

use crate::SynapseType;

#[derive(Component, Debug)]
pub struct SimpleSynapse {
    pub weight: f64,
    pub delay: u32,
    pub source: Entity,
    pub target: Entity,
    pub synapse_type: SynapseType,
}
