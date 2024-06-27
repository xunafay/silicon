use bevy::prelude::ResMut;
use silicon_core::Clock;

pub(crate) fn update_clock(mut clock: ResMut<Clock>) {
    clock.time += clock.tau;
}
