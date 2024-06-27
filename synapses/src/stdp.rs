use bevy::prelude::{Component, Entity, Resource};

use crate::{Synapse, SynapseType};

#[derive(Debug, Resource)]
pub struct StdpSettings {
    pub look_back: f64,
    pub update_interval: f64,
    pub next_update: f64,
}

#[derive(Debug, Component)]
pub struct StdpSynapse {
    pub weight: f64,
    pub delay: u32,
    pub source: Entity,
    pub target: Entity,
    pub synapse_type: SynapseType,
    pub stdp_params: StdpParams,
}

#[derive(Debug, Clone)]
pub struct StdpParams {
    pub a_plus: f64,
    pub a_minus: f64,
    pub tau_plus: f64,
    pub tau_minus: f64,
    pub w_max: f64,
    pub w_min: f64,
    pub synapse_type: SynapseType,
}

impl Synapse for StdpSynapse {
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
