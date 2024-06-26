use bevy::{
    app::{App, Plugin, Update},
    prelude::{Entity, Event},
};
use time::{update_clock, Clock};
pub mod time;

#[derive(Event, Debug)]
pub struct SpikeEvent {
    pub time: f64,
    pub neuron: Entity,
}

#[derive(Debug)]
pub struct Spike {
    pub time: f64,
    pub neuron: Entity,
}

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Clock {
            time: 0.0,
            tau: 0.025,
        })
        .add_event::<SpikeEvent>()
        .add_systems(Update, update_clock);
    }
}
