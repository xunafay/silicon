use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{
    egui::{self, Color32},
    EguiContext, EguiContexts, EguiPlugin,
};
use egui_plot::{Legend, Line, Plot, VLine};
use uom::si::{f64::Time, time::second};

use crate::{data::MembranePlotter, neurons::Clock, Insights};

pub struct SiliconUiPlugin;

impl Plugin for SiliconUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_plugins(bevy_inspector_egui::DefaultInspectorConfigPlugin) // adds default options and `InspectorEguiImpl`s
            .add_systems(Update, (neuron_inspect_window, inspector_ui))
            .insert_resource(SimulationUiState {
                simulation_time_slider: 50.0,
            });
    }
}

#[derive(Resource, Debug)]
pub struct SimulationUiState {
    simulation_time_slider: f64,
}

fn neuron_inspect_window(
    mut clock: ResMut<Clock>,
    insights: Res<Insights>,
    mut state: ResMut<SimulationUiState>,
    mut contexts: EguiContexts,
    plotters: Query<(Entity, &MembranePlotter)>,
) {
    let selected_plotter = plotters.iter().find(|(entity, _)| {
        insights
            .selected_entity
            .map_or(false, |selected_entity| *entity == selected_entity)
    });
    egui::Window::new("Simulation").show(contexts.ctx_mut(), |ui| {
        ui.label(format!("Time: {:.2}", clock.time));
        ui.add(
            egui::Slider::new(&mut state.simulation_time_slider, 0.0..=100.0)
                .clamp_to_range(false)
                .text("Time to simulate in ms"),
        );

        let button = ui
            .button("Run")
            .on_hover_text("Run the simulation for the specified time");

        ui.add(
            egui::Slider::new(&mut clock.tau, 0.001..=0.1)
                .clamp_to_range(false)
                .text("Time constant in ms"),
        );

        if button.clicked() {
            info!("Running simulation for {} ms", state.simulation_time_slider);
        }
    });

    egui::Window::new("Neuron Inspector").show(contexts.ctx_mut(), |ui| {
        if insights.selected_entity.is_none() {
            ui.label("No neuron selected");
            return;
        }

        let plot = Plot::new("Test").legend(Legend::default());

        if let Some((entity, plotter)) = selected_plotter {
            plot.show(ui, |plot_ui| {
                let spikes = plotter
                    .spike_lines(Time::new::<second>(100.0), Time::new::<second>(clock.time));
                for spike in spikes {
                    plot_ui.vline(VLine::new(spike).color(Color32::RED));
                }

                plot_ui.line(
                    Line::new(
                        plotter.plot_points(
                            Time::new::<second>(100.0),
                            Time::new::<second>(clock.time),
                        ),
                    )
                    .name(format!("{:?}", entity))
                    .color(Color32::BLUE),
                );
            });
        }
    });
}

fn inspector_ui(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    egui::Window::new("Inspector")
        .default_open(false)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // equivalent to `WorldInspectorPlugin`
                bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);

                egui::CollapsingHeader::new("Materials").show(ui, |ui| {
                    bevy_inspector_egui::bevy_inspector::ui_for_assets::<StandardMaterial>(
                        world, ui,
                    );
                });

                ui.heading("Entities");
                bevy_inspector_egui::bevy_inspector::ui_for_world_entities(world, ui);
            });
        });
}
