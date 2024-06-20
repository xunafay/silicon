use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContext, EguiContexts, EguiPlugin};
use egui_plot::{Legend, Line, Plot};

use crate::{data::MembranePlotter, neurons::Clock, Insights};

pub struct SiliconUiPlugin;

impl Plugin for SiliconUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_plugins(bevy_inspector_egui::DefaultInspectorConfigPlugin) // adds default options and `InspectorEguiImpl`s
            .add_systems(Update, (neuron_inspect_window, inspector_ui));
    }
}

fn neuron_inspect_window(
    clock: Res<Clock>,
    insights: Res<Insights>,
    mut contexts: EguiContexts,
    plotters: Query<(Entity, &MembranePlotter)>,
) {
    let selected_plotter = plotters.iter().find(|(entity, _)| {
        insights
            .selected_entity
            .map_or(false, |selected_entity| *entity == selected_entity)
    });
    egui::Window::new("Info").show(contexts.ctx_mut(), |ui| {
        ui.label(format!("Time: {:.2}", clock.time));
        let plot = Plot::new("Test").legend(Legend::default());

        if let Some((entity, plotter)) = selected_plotter {
            plot.show(ui, |plot_ui| {
                plot_ui.line(Line::new(plotter.plot_points()).name(format!("{:?}", entity)));
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
