use bevy::prelude::ResMut;
use silicon_core::Clock;

pub(crate) fn update_clock(mut clock: ResMut<Clock>) {
    if clock.run_indefinitely && clock.time_to_simulate <= 0.1 {
        clock.time_to_simulate += 0.1;
    }

    if clock.time_to_simulate <= 0.0 {
        return;
    }

    clock.time += clock.tau;
    clock.time_to_simulate -= clock.tau;
}
