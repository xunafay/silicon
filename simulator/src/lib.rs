use bevy::prelude::Resource;

#[derive(Resource)]
pub struct Clock {
    pub time: f64,
    pub tau: f64,
}
