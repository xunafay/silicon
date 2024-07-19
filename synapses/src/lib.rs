use bevy::{
    app::{App, Plugin, Update},
    prelude::{Component, Entity, Event, Events, Query, Res, ResMut, Resource},
    reflect::Reflect,
};
use bevy_trait_query::{One, RegisterExt};
use silicon_core::Clock;
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

/// A resource that configures the decay of synapses.
/// Add this resource to the App to enable synapse decay.
/// substracts the amount from the weight of all synapses at the interval.
#[derive(Debug, Clone, Reflect, Resource)]
pub struct SynapseDecay {
    pub interval: f64,
    pub amount: f64,
    pub next_decay: f64,
}

fn decay_synapses(
    mut synapses: Query<One<&mut dyn Synapse>>,
    time: Res<Clock>,
    mut decay: Option<ResMut<SynapseDecay>>,
) {
    if let Some(decay) = decay.as_mut() {
        let time = time.time;
        if time >= decay.next_decay {
            decay.next_decay = time + decay.interval;
            for mut synapse in synapses.iter_mut() {
                let weight = synapse.get_weight();
                synapse.set_weight(weight - decay.amount);
            }
        }
    }
}

pub struct SynapsePlugin;

impl Plugin for SynapsePlugin {
    fn build(&self, app: &mut App) {
        app.register_component_as::<dyn Synapse, SimpleSynapse>()
            .register_component_as::<dyn Synapse, StdpSynapse>()
            .register_type::<SimpleSynapse>()
            .register_type::<StdpSynapse>()
            .init_resource::<Events<DeferredStdpEvent>>()
            .add_systems(Update, decay_synapses);
    }
}
