use bevy::{
    color::{Color, LinearRgba},
    prelude::Component,
    reflect::Reflect,
};

#[derive(Component, Debug, PartialEq, Clone, Copy, Reflect)]
pub enum ColumnLayer {
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
}

impl ColumnLayer {
    pub fn get_color(&self) -> Color {
        match self {
            ColumnLayer::L1 => Color::srgb(0.0, 0.0, 1.0),
            ColumnLayer::L2 => Color::srgb(0.0, 0.5, 1.0),
            ColumnLayer::L3 => Color::srgb(0.0, 1.0, 1.0),
            ColumnLayer::L4 => Color::srgb(0.5, 1.0, 0.5),
            ColumnLayer::L5 => Color::srgb(1.0, 1.0, 0.0),
            ColumnLayer::L6 => Color::srgb(1.0, 0.5, 0.0),
        }
    }

    pub fn get_color_from_activation(&self, activation_percentage: f64) -> LinearRgba {
        let color = self.get_color();
        LinearRgba::rgb(
            refit_to_range(
                activation_percentage as f32,
                0.0,
                1.0,
                0.0,
                color.to_linear().red * 5.0,
            ),
            refit_to_range(
                activation_percentage as f32,
                0.0,
                1.0,
                0.0,
                color.to_linear().green * 5.0,
            ),
            refit_to_range(
                activation_percentage as f32,
                0.0,
                1.0,
                0.0,
                color.to_linear().blue * 5.0,
            ),
        )
    }
}

fn refit_to_range(n: f32, start1: f32, stop1: f32, start2: f32, stop2: f32) -> f32 {
    ((n - start1) / (stop1 - start1)) * (stop2 - start2) + start2
}
