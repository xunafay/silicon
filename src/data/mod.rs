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
}

#[derive(Debug)]
pub struct MembranePlotPoint {
    pub potential: f64,
    pub time: f64,
}

impl MembranePlotter {
    pub fn new() -> Self {
        MembranePlotter { points: Vec::new() }
    }

    pub fn add_point(&mut self, potential: f64, time: f64) {
        self.points.push(MembranePlotPoint { potential, time });
    }

    pub fn plot_points(&self) -> PlotPoints {
        let mut points: Vec<[f64; 2]> = Vec::new();
        for point in &self.points {
            points.push([point.time, point.potential]);
        }
        PlotPoints::new(points)
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
