use bevy::{prelude::*, render::camera::Viewport, window::PrimaryWindow};
use bevy_egui::{EguiContext, EguiPlugin, EguiSet};
use state::UiState;
use transform_gizmo_egui::GizmoMode;

pub struct SiliconUiPlugin;

pub mod state;

impl Plugin for SiliconUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_plugins(bevy_inspector_egui::DefaultInspectorConfigPlugin) // adds default options and `InspectorEguiImpl`s
            // .add_systems(Update, (neuron_inspect_window, inspector_ui))
            .add_systems(
                PostUpdate,
                (
                    show_ui_system
                        .before(EguiSet::ProcessOutput)
                        .before(bevy::transform::TransformSystem::TransformPropagate),
                    set_camera_viewport.after(show_ui_system),
                ),
            )
            .add_systems(Update, set_gizmo_mode)
            .insert_resource(SimulationUiState {
                simulation_time_slider: 50.0,
            })
            .insert_resource(UiState::new());
    }
}

#[derive(Resource, Debug)]
pub struct SimulationUiState {
    simulation_time_slider: f64,
}

fn show_ui_system(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    world.resource_scope::<UiState, _>(|world, mut ui_state| {
        ui_state.ui(world, egui_context.get_mut())
    });
}

// make camera only render to view not obstructed by UI
fn set_camera_viewport(
    ui_state: Res<UiState>,
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    egui_settings: Res<bevy_egui::EguiSettings>,
    mut cameras: Query<&mut Camera>,
) {
    let mut cam = cameras.single_mut();

    let Ok(window) = primary_window.get_single() else {
        return;
    };

    let scale_factor = window.scale_factor() * egui_settings.scale_factor;

    let viewport_pos = ui_state.viewport_rect.left_top().to_vec2() * scale_factor;
    let viewport_size = ui_state.viewport_rect.size() * scale_factor;

    cam.viewport = Some(Viewport {
        physical_position: UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32),
        physical_size: UVec2::new(viewport_size.x as u32, viewport_size.y as u32),
        depth: 0.0..1.0,
    });
}

fn set_gizmo_mode(input: Res<ButtonInput<KeyCode>>, mut ui_state: ResMut<UiState>) {
    for (key, mode) in [
        (KeyCode::KeyR, GizmoMode::RotateX),
        (KeyCode::KeyT, GizmoMode::TranslateXY),
        (KeyCode::KeyS, GizmoMode::ScaleXY),
    ] {
        if input.just_pressed(key) {
            ui_state.gizmo_mode = mode;
        }
    }
}
