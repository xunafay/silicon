use bevy::{
    prelude::{Bundle, Component},
    render::color::Color,
};

#[derive(Component, Debug)]
pub struct MacroColumn;

#[derive(Component, Debug)]
pub struct MiniColumn;

#[derive(Component, Debug)]
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
            ColumnLayer::L1 => Color::rgb(0.0, 0.0, 1.0),
            ColumnLayer::L2 => Color::rgb(0.0, 0.5, 1.0),
            ColumnLayer::L3 => Color::rgb(0.0, 1.0, 1.0),
            ColumnLayer::L4 => Color::rgb(0.5, 1.0, 0.5),
            ColumnLayer::L5 => Color::rgb(1.0, 1.0, 0.0),
            ColumnLayer::L6 => Color::rgb(1.0, 0.5, 0.0),
        }
    }

    pub fn get_color_from_potential(
        &self,
        membrane_potential: f32,
        resting_potential: f32,
        threshold_potential: f32,
    ) -> Color {
        let color = self.get_color();
        Color::rgb_linear(
            refit_to_range(
                membrane_potential,
                resting_potential,
                threshold_potential,
                0.0,
                color.r() * 2000.0,
            ),
            refit_to_range(
                membrane_potential,
                resting_potential,
                threshold_potential,
                0.0,
                color.g() * 2000.0,
            ),
            refit_to_range(
                membrane_potential,
                resting_potential,
                threshold_potential,
                0.0,
                color.b() * 2000.0,
            ),
        )
    }
}

fn refit_to_range(n: f32, start1: f32, stop1: f32, start2: f32, stop2: f32) -> f32 {
    ((n - start1) / (stop1 - start1)) * (stop2 - start2) + start2
}

#[derive(Bundle, Debug)]
struct MiniColumnBundle {
    mini_column: MiniColumn,
    layer: ColumnLayer,
}
