use bevy::{
    app::{App, Plugin},
    prelude::{Component, Entity, Event, Events},
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
    fn update(&mut self, tau: f64);

    fn get_weight(&self) -> f64;
    fn set_weight(&mut self, weight: f64);

    fn get_presynaptic(&self) -> Entity;
    fn get_postsynaptic(&self) -> Entity;

    fn get_type(&self) -> SynapseType;
}

#[derive(Debug, PartialEq, Copy, Clone, Default, Reflect)]
pub enum SynapseType {
    #[default]
    Excitatory,
    Inhibitory,
}

/// The primary purpose of this event is to allow for reward modulated STDP. By deferring the
/// weight update, the reward signal can be used to determine the modify the delta_weight value
/// before the weight is updated.
///
/// This event does not get cleaned up automatically. It is up to the user to ensure that the
/// event is cleaned up after it is no longer needed.
#[derive(Debug, PartialEq, Copy, Clone, Reflect, Event)]
pub struct DeferredStdpEvent {
    pub synapse: Entity,
    pub delta_weight: f64,
}

pub struct SynapsePlugin;

impl Plugin for SynapsePlugin {
    fn build(&self, app: &mut App) {
        app.register_component_as::<dyn Synapse, SimpleSynapse>()
            .register_component_as::<dyn Synapse, StdpSynapse>()
            .register_type::<SimpleSynapse>()
            .register_type::<StdpSynapse>()
            .init_resource::<Events<DeferredStdpEvent>>();
    }
}
