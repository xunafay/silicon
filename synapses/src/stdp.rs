use bevy::{
    log::trace,
    prelude::{Component, Entity, Resource},
    reflect::Reflect,
};

use crate::{Synapse, SynapseType};

#[derive(Debug, Resource, Reflect)]
pub struct StdpSettings {
    pub look_back: f64,
    pub update_interval: f64,
    pub next_update: f64,
}

#[derive(Debug, Component, Reflect)]
pub struct StdpSynapse {
    pub weight: f64,
    pub delay: u32,
    pub source: Entity,
    pub target: Entity,
    pub synapse_type: SynapseType,
    pub stdp_params: StdpParams,
    pub stdp_state: StdpState,
}

#[derive(Debug, Clone, Reflect)]
pub struct StdpState {
    pub a: f64,
    pub spike_type: StdpSpikeType,
}

#[derive(Debug, Clone, Reflect, PartialEq, Eq)]
pub enum StdpSpikeType {
    PreSpike,
    PostSpike,
}

#[derive(Debug, Clone, Reflect)]
pub struct StdpParams {
    /// the maximum value of a positive weight change
    pub a_plus: f64,
    /// the maximum value of a negative weight change
    pub a_minus: f64,
    /// the time constant for the decay of the positive weight change
    pub tau_plus: f64,
    /// the time constant for the decay of the negative weight change
    pub tau_minus: f64,
    /// the maximum value of the weight
    pub w_max: f64,
    /// the minimum value of the weight
    pub w_min: f64,
}

impl StdpSynapse {
    pub fn register_pre_spike(&mut self) {
        if self.stdp_state.a.abs() > f64::EPSILON
            && self.stdp_state.spike_type == StdpSpikeType::PostSpike
        {
            trace!("Updating weight: {} because we registered an incoming spike after an outgoing spike", self.weight);
            self.weight = (self.weight + self.stdp_state.a)
                .min(self.stdp_params.w_max)
                .max(self.stdp_params.w_min);
        }

        self.stdp_state.spike_type = StdpSpikeType::PreSpike;
        self.stdp_state.a = self.stdp_params.a_plus;
    }

    pub fn register_post_spike(&mut self) {
        if self.stdp_state.a.abs() > f64::EPSILON
            && self.stdp_state.spike_type == StdpSpikeType::PreSpike
        {
            trace!("Updating weight: {} because we registered an outgoing spike after an incoming spike", self.weight);
            self.weight = (self.weight - self.stdp_state.a)
                .min(self.stdp_params.w_max)
                .max(self.stdp_params.w_min);
        }

        self.stdp_state.spike_type = StdpSpikeType::PostSpike;
        self.stdp_state.a = self.stdp_params.a_minus;
    }
}

impl Synapse for StdpSynapse {
    fn update(&mut self, tau: f64) {
        let delta_a = match self.stdp_state.spike_type {
            StdpSpikeType::PreSpike => (0.0 - self.stdp_state.a) * tau,
            StdpSpikeType::PostSpike => (0.0 - self.stdp_state.a) * tau,
        };

        self.stdp_state.a += delta_a;
    }

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
