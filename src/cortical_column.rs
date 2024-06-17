use bevy::prelude::Component;

pub enum CorticalColumnLayer {
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
}

#[derive(Debug, Component)]
pub struct CorticalColumn {
    pub x: i64,
    pub y: i64,
}
