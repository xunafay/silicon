use bevy::prelude::{ResMut, Resource};

pub(crate) fn update_clock(mut clock: ResMut<Clock>) {
    clock.time += clock.tau;
}

#[derive(Resource)]
pub struct Clock {
    pub time: f64,
    pub tau: f64,
}
