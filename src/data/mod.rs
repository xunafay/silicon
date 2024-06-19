use bevy::prelude::*;
use egui_plot::PlotPoints;

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
