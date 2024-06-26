use bevy::prelude::Resource;

#[derive(Debug, Clone, Resource)]
pub struct Dopamine {
    pub amount: f64,
}
