use bevy::{
    app::{App, Update},
    prelude::{Component, Plugin, Query, Res},
};
use bevy_trait_query::One;
use egui_plot::PlotPoints;
use neurons::Neuron;
use simulator::time::Clock;

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

    pub fn plot_points(&self, time_span: f64, current_time: f64) -> PlotPoints {
        let points: Vec<[f64; 2]> = self
            .points
            .iter()
            .filter(|point| point.time >= current_time - time_span)
            .map(|point| [point.time, point.potential])
            .collect();
        PlotPoints::new(points)
    }

    pub fn spike_lines(&self, time_span: f64, current_time: f64) -> Vec<f64> {
        self.spikes
            .iter()
            .filter(|time| **time >= current_time - time_span)
            .copied()
            .collect()
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
