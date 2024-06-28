use bevy::{
    app::{App, Plugin},
    prelude::{Component, Entity},
    reflect::Reflect,
};
use bevy_trait_query::RegisterExt;
use simple::SimpleSynapse;
use stdp::StdpSynapse;

pub mod simple;
pub mod stdp;

/// A component that allows a neuron to receive synapses.
#[derive(Component, Debug, Reflect)]
pub struct AllowSynapses;

#[bevy_trait_query::queryable]
pub trait Synapse {
    fn get_weight(&self) -> f64;
    fn set_weight(&mut self, weight: f64);

    fn get_presynaptic(&self) -> Entity;
    fn get_postsynaptic(&self) -> Entity;

    fn get_type(&self) -> SynapseType;
}

#[derive(Debug, Copy, Clone, Default, Reflect)]
pub enum SynapseType {
    #[default]
    Excitatory,
    Inhibitory,
}

pub struct SynapsePlugin;

impl Plugin for SynapsePlugin {
    fn build(&self, app: &mut App) {
        app.register_component_as::<dyn Synapse, SimpleSynapse>()
            .register_component_as::<dyn Synapse, StdpSynapse>()
            .register_type::<SimpleSynapse>()
            .register_type::<StdpSynapse>();
    }
}
