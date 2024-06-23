use bevy::{
    app::{App, Update},
    prelude::{Component, Plugin, Query, Res},
};
use egui_plot::PlotPoints;
use uom::si::{electric_potential::millivolt, f64::Time, time::second};

use crate::neurons::{Clock, Neuron};

pub struct NeuronDataCollectionPlugin;

impl Plugin for NeuronDataCollectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_plotters);
    }
}

#[derive(Debug, Component)]
pub struct MembranePlotter {
    pub points: Vec<MembranePlotPoint>,
    pub spikes: Vec<f64>,
}

#[derive(Debug)]
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

    pub fn plot_points(&self, time_span: Time, current_time: Time) -> PlotPoints {
        let points: Vec<[f64; 2]> = self
            .points
            .iter()
            .filter(|point| point.time >= current_time.get::<second>() - time_span.get::<second>())
            .map(|point| [point.time, point.potential])
            .collect();
        PlotPoints::new(points)
    }

    pub fn spike_lines(&self, time_span: Time, current_time: Time) -> Vec<f64> {
        self.spikes
            .iter()
            .filter(|time| **time >= current_time.get::<second>() - time_span.get::<second>())
            .copied()
            .collect()
    }
}

fn update_plotters(mut plotter_query: Query<(&Neuron, &mut MembranePlotter)>, clock: Res<Clock>) {
    for (neuron, mut membrane_plotter) in plotter_query.iter_mut() {
        membrane_plotter.add_point(
            neuron.membrane_potential.get::<millivolt>(),
            Time::new::<second>(clock.time).get::<second>(),
        );
    }
}
