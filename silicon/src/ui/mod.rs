use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode,
    },
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    render::camera::Viewport,
    window::PrimaryWindow,
};
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
            .add_systems(Startup, setup)
            .add_systems(Update, mouse_scroll)
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

#[derive(Component)]
pub struct MainCamera {}

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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    // commands.spawn((Camera2dBundle::default(), IsDefaultUiCamera));

    // root node
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // left vertical fill (border)
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(200.),
                        border: UiRect::all(Val::Px(2.)),
                        ..default()
                    },
                    background_color: Color::rgb(0.65, 0.65, 0.65).into(),
                    ..default()
                })
                .with_children(|parent| {
                    // left vertical fill (content)
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.),
                                ..default()
                            },
                            background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            // text
                            parent.spawn((
                                TextBundle::from_section(
                                    "Text Example",
                                    TextStyle {
                                        font: asset_server
                                            .load("fonts/JetBrainsMonoNL-Regular.ttf"),
                                        font_size: 30.0,
                                        ..default()
                                    },
                                )
                                .with_style(Style {
                                    margin: UiRect::all(Val::Px(5.)),
                                    ..default()
                                }),
                                // Because this is a distinct label widget and
                                // not button/list item text, this is necessary
                                // for accessibility to treat the text accordingly.
                                Label,
                            ));
                        });
                });
            // right vertical fill
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        width: Val::Px(200.),
                        ..default()
                    },
                    background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                    ..default()
                })
                .with_children(|parent| {
                    // Title
                    parent.spawn((
                        TextBundle::from_section(
                            "Scrolling list",
                            TextStyle {
                                font: asset_server.load("fonts/JetBrainsMonoNL-Regular.ttf"),
                                font_size: 25.,
                                ..default()
                            },
                        ),
                        Label,
                    ));
                    // List with hidden overflow
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                align_self: AlignSelf::Stretch,
                                height: Val::Percent(50.),
                                overflow: Overflow::clip_y(),
                                ..default()
                            },
                            background_color: Color::rgb(0.10, 0.10, 0.10).into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            // Moving panel
                            parent
                                .spawn((
                                    NodeBundle {
                                        style: Style {
                                            flex_direction: FlexDirection::Column,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        ..default()
                                    },
                                    ScrollingList::default(),
                                    AccessibilityNode(NodeBuilder::new(Role::List)),
                                ))
                                .with_children(|parent| {
                                    // List items
                                    for i in 0..30 {
                                        parent.spawn((
                                            TextBundle::from_section(
                                                format!("Item {i}"),
                                                TextStyle {
                                                    font: asset_server
                                                        .load("fonts/JetBrainsMonoNL-Regular.ttf"),
                                                    font_size: 20.,
                                                    ..default()
                                                },
                                            ),
                                            Label,
                                            AccessibilityNode(NodeBuilder::new(Role::ListItem)),
                                        ));
                                    }
                                });
                        });
                });
        });
}

#[derive(Component, Default)]
struct ScrollingList {
    position: f32,
}

fn mouse_scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query_list: Query<(&mut ScrollingList, &mut Style, &Parent, &Node)>,
    query_node: Query<&Node>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        for (mut scrolling_list, mut style, parent, list_node) in &mut query_list {
            let items_height = list_node.size().y;
            let container_height = query_node.get(parent.get()).unwrap().size().y;

            let max_scroll = (items_height - container_height).max(0.);

            let dy = match mouse_wheel_event.unit {
                MouseScrollUnit::Line => mouse_wheel_event.y * 20.,
                MouseScrollUnit::Pixel => mouse_wheel_event.y,
            };

            scrolling_list.position += dy;
            scrolling_list.position = scrolling_list.position.clamp(-max_scroll, 0.);
            style.top = Val::Px(scrolling_list.position);
        }
    }
}

// make camera only render to view not obstructed by UI
fn set_camera_viewport(
    ui_state: Res<UiState>,
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    egui_settings: Res<bevy_egui::EguiSettings>,
    mut cameras: Query<&mut Camera, With<MainCamera>>,
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
