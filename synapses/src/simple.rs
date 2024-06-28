use bevy::{
    prelude::{Component, Entity},
    reflect::Reflect,
};

use crate::{Synapse, SynapseType};

#[derive(Component, Debug, Reflect)]
pub struct SimpleSynapse {
    pub weight: f64,
    pub delay: u32,
    pub source: Entity,
    pub target: Entity,
    pub synapse_type: SynapseType,
}

impl Synapse for SimpleSynapse {
    fn get_weight(&self) -> f64 {
        self.weight
    }

    fn set_weight(&mut self, weight: f64) {
        self.weight = weight;
    }

    fn get_presynaptic(&self) -> Entity {
        self.source
    }

    fn get_postsynaptic(&self) -> Entity {
        self.target
    }

    fn get_type(&self) -> SynapseType {
        self.synapse_type
    }
}
