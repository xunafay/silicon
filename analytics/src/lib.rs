use bevy::{
    app::{App, Plugin, Update},
    prelude::{Component, Query, Res},
    reflect::Reflect,
};
use bevy_trait_query::One;
use silicon_core::{Clock, Neuron};

pub struct SiliconAnalyticsPlugin;

impl Plugin for SiliconAnalyticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_plotters)
            .register_type::<MembranePlotter>()
            .register_type::<MembranePlotPoint>();
    }
}

fn update_plotters(
    mut plotter_query: Query<(One<&dyn Neuron>, &mut MembranePlotter)>,
    clock: Res<Clock>,
) {
    for (neuron, mut membrane_plotter) in plotter_query.iter_mut() {
        membrane_plotter.add_point(neuron.get_membrane_potential(), clock.time);
    }
}

#[derive(Debug, Component, Reflect)]
pub struct MembranePlotter {
    pub points: Vec<MembranePlotPoint>,
    pub spikes: Vec<f64>,
}

#[derive(Debug, Reflect)]
pub struct MembranePlotPoint {
    pub potential: f64,
    pub time: f64,
}

impl MembranePlotter {
    pub fn new() -> Self {
        MembranePlotter {
            points: Vec::new(),
            spikes: Vec::new(),
        }
    }

    pub fn add_point(&mut self, potential: f64, time: f64) {
        self.points.push(MembranePlotPoint { potential, time });
    }

    pub fn add_spike(&mut self, time: f64) {
        self.spikes.push(time);
    }

    pub fn plot_points(&self, time_span: f64, current_time: f64) -> Vec<[f64; 2]> {
        self.points
            .iter()
            .filter(|point| point.time >= current_time - time_span)
            .map(|point| [point.time, point.potential])
            .collect()
    }

    pub fn spike_lines(&self, time_span: f64, current_time: f64) -> Vec<f64> {
        self.spikes
            .iter()
            .filter(|time| **time >= current_time - time_span)
            .copied()
            .collect()
    }
}

impl Default for MembranePlotter {
    fn default() -> Self {
        Self::new()
    }
}
