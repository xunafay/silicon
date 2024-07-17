use bevy::{
    asset::{AssetServer, Assets},
    color::Color,
    math::Vec2,
    prelude::{Commands, Res, ResMut},
    sprite::{ColorMaterial, Sprite, SpriteBundle},
};

pub struct PlotData {
    pub lines: Vec<PlotLine>,
    pub hlines: Vec<f32>,
    pub vlines: Vec<f32>,
}

pub struct PlotLine {
    pub points: Vec<(f32, f32)>,
    pub color: Color,
    pub width: f32,
    pub label: Option<String>,
}

pub struct PlotConfig {
    pub title: Option<String>,
    pub x_label: Option<String>,
    pub y_label: Option<String>,
    pub width: f32,
    pub height: f32,
}

pub struct Plot {
    pub data: PlotData,
    pub config: PlotConfig,
}

impl Plot {
    pub fn new(data: PlotData, config: PlotConfig) -> Self {
        Plot { data, config }
    }

    pub fn add_line(&mut self, line: PlotLine) {
        self.data.lines.push(line);
    }

    pub fn add_hline(&mut self, y: f32) {
        self.data.hlines.push(y);
    }

    pub fn add_vline(&mut self, x: f32) {
        self.data.vlines.push(x);
    }
}
