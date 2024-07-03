#![allow(clippy::type_complexity)]

use std::time::Duration;

use analytics::SiliconAnalyticsPlugin;
use bevy::{
    core::TaskPoolThreadAssignmentPolicy,
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings},
        tonemapping::Tonemapping,
    },
    diagnostic::FrameTimeDiagnosticsPlugin,
    log::LogPlugin,
    pbr::ClusterConfig,
    prelude::*,
    tasks::available_parallelism,
    window::WindowResolution,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_rapier3d::{
    pipeline::QueryFilter,
    plugin::{NoUserData, RapierContext, RapierPhysicsPlugin},
};
use bevy_trait_query::One;
use neurons::NeuronPlugin;
use rand::Rng;
use silicon_core::{Clock, Neuron, NeuronVisualizer};
use simulator::SimulationPlugin;
use structure::{cortical_column::MiniColumn, layer::ColumnLayer, test_column::TestColumn};
use synapses::{
    simple::SimpleSynapse,
    stdp::{StdpParams, StdpSynapse},
    Synapse, SynapsePlugin, SynapseType,
};
use transcoder::nlp::string_to_spike_train;
use ui::{state::UiState, SiliconUiPlugin};

mod structure;
mod ui;

fn main() {
    App::new().add_plugins(SiliconPlugin).run();
}

#[derive(Resource)]
pub struct Insights {
    pub selected_entity: Option<Entity>,
}

pub struct SiliconPlugin;

impl Plugin for SiliconPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    level: bevy::log::Level::TRACE,
                    filter: "info,silicon=trace,simulator=trace".into(),
                    ..Default::default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Neuron Simulation".to_string(),
                        resolution: WindowResolution::new(1920.0, 1080.0),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(TaskPoolPlugin {
                    task_pool_options: TaskPoolOptions {
                        // this thread setup is optimized for a compute-heavy workload and not asset loading
                        compute: TaskPoolThreadAssignmentPolicy {
                            min_threads: available_parallelism(),
                            max_threads: usize::MAX,
                            percent: 1.0,
                        },
                        ..default()
                    },
                }),
        )
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(SiliconUiPlugin)
        .add_plugins((
            SimulationPlugin,
            NeuronPlugin,
            SynapsePlugin,
            SiliconAnalyticsPlugin,
        ))
        .add_plugins(FrameTimeDiagnosticsPlugin)
        // .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(Msaa::Sample8)
        .insert_resource(Insights {
            selected_entity: None,
        })
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(5000)))
        .insert_resource(EncoderState::default())
        .add_systems(Update, insert_current)
        .add_systems(PostStartup, notify_setup_done)
        .add_systems(Update, show_select_neuron_synapses)
        .add_systems(Update, (update_neuron_materials, mouse_click))
        .add_systems(
            Startup,
            ((create_neurons, create_synapses).chain(), setup_scene),
        );
        // .add_systems(PostStartup, hide_meshes) // hide meshes if you need some extra performance
    }
}

#[allow(dead_code)]
fn hide_meshes(mut visibilities: Query<&mut Visibility>) {
    for mut visibility in visibilities.iter_mut() {
        *visibility = Visibility::Hidden;
    }
}

// Inherited visibilty didn't work for me, so I had to query the children and set their visibility too
fn show_select_neuron_synapses(
    insights: Res<Insights>,
    mut synapse_query: Query<(One<&dyn Synapse>, &mut Visibility, &Children)>,
    mut child_query: Query<&mut Visibility, (Without<StdpSynapse>, Without<SimpleSynapse>)>, // https://github.com/JoJoJet/bevy-trait-query/pull/58
) {
    if let Some(selected_entity) = insights.selected_entity {
        for (synapse, mut visibility, children) in synapse_query.iter_mut() {
            let is_visible = synapse.get_presynaptic() == selected_entity
                || synapse.get_postsynaptic() == selected_entity;

            *visibility = if is_visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };

            // Update the visibility of its children
            for &child in children.iter() {
                if let Ok(mut child_visibility) = child_query.get_mut(child) {
                    *child_visibility = if is_visible {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    };
                }
            }
        }
    } else {
        for (_, mut visibility, children) in synapse_query.iter_mut() {
            *visibility = Visibility::Visible;

            // Update the visibility of its children
            for &child in children.iter() {
                if let Ok(mut child_visibility) = child_query.get_mut(child) {
                    *child_visibility = Visibility::Visible;
                }
            }
        }
    }
}

fn notify_setup_done() {
    info!("Setup done!");
}

#[derive(Debug, Resource, Reflect)]
struct EncoderState {
    pub is_playing: bool,
    pub current_time: f64,
    pub paused_time: f64,
    pub spike_train: Vec<f64>,
}

impl Default for EncoderState {
    fn default() -> Self {
        EncoderState {
            is_playing: false,
            current_time: 0.0,
            paused_time: 5.0,
            spike_train: Vec::new(),
        }
    }
}

fn insert_current(
    mut neurons_query: Query<(Entity, One<&mut dyn Neuron>, &ColumnLayer)>,
    clock: Res<Clock>,
    mut encoder: ResMut<EncoderState>,
) {
    if clock.time_to_simulate <= 0.0 {
        return;
    }

    if encoder.is_playing {
        encoder.current_time += clock.tau;
        let last = encoder.spike_train.last();
        if let Some(last) = last {
            if last <= &encoder.current_time {
                encoder.spike_train.pop();
                for (_, mut neuron, layer) in neurons_query.iter_mut() {
                    if layer != &ColumnLayer::L4 {
                        continue;
                    }

                    neuron.add_membrane_potential(rand::thread_rng().gen_range(0.4..=0.8));
                }
            }
        } else {
            trace!("End of spike train");
            encoder.is_playing = false;
            encoder.paused_time = 5.0;
            encoder.current_time = 0.0;
        }
    } else {
        encoder.paused_time -= clock.tau;
        if encoder.paused_time <= 0.0 {
            encoder.is_playing = true;
            encoder.spike_train = string_to_spike_train("hello", 5.0);
            encoder.spike_train.reverse();
            trace!("Playing spike train: {:?}", encoder.spike_train);
        }
    }
}

fn create_neurons(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    MiniColumn::create(commands, meshes, materials);
}

fn create_synapses(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    neuron_query: Query<(Entity, &mut dyn Neuron, &Transform)>,
) {
    trace!(
        "Creating synapses for {} neurons",
        neuron_query.iter().len()
    );

    let synapse_material_excitory = materials.add(StandardMaterial {
        base_color: Color::rgba(0.4, 0.4, 1.0, 0.8),
        emissive: Color::rgb_linear(0.3, 0.3, 200.0), // Bright green emissive color
        alpha_mode: AlphaMode::Blend,                 // Enable blending for translucency
        ..Default::default()
    });

    let synapse_material_inhibitory = materials.add(StandardMaterial {
        base_color: Color::rgba(1.0, 0.4, 0.4, 0.8),
        emissive: Color::rgb_linear(200.0, 0.3, 0.3), // Bright red emissive color
        alpha_mode: AlphaMode::Blend,                 // Enable blending for translucency
        ..Default::default()
    });

    let mut iter = neuron_query.iter_combinations();

    while let Some([(pre_entity, _, pre_transform), (post_entity, _, post_transform)]) =
        iter.fetch_next()
    {
        if rand::random::<f64>() < 0.8 {
            continue;
        }

        let midpoint = (pre_transform.translation + post_transform.translation) / 2.0;
        let synapse_pos_pre =
            (pre_transform.translation + midpoint) / 2.0 - pre_transform.translation;
        let synapse_pos_post =
            (post_transform.translation + midpoint) / 2.0 - pre_transform.translation;
        let direction = post_transform.translation - pre_transform.translation;
        let length = direction.length();
        let normalized_direction = direction.normalize();
        let rotation = Quat::from_rotation_arc(Vec3::Y, normalized_direction);

        let synapse_stalk_mesh = meshes.add(Capsule3d::new(0.05, length).mesh());
        let synapse_mesh = meshes.add(
            Cylinder {
                half_height: 0.2,
                radius: 0.2,
            }
            .mesh(),
        );

        let synapse_type = if rand::random::<f64>() > 0.2 {
            SynapseType::Excitatory
        } else {
            SynapseType::Inhibitory
        };

        let synapse_direction = rand::random::<f64>() > 0.5;

        let synapse = commands
            .spawn((
                StdpSynapse {
                    stdp_params: StdpParams {
                        a_plus: 0.01,
                        a_minus: -0.01,
                        tau_plus: 0.02,
                        tau_minus: 0.02,
                        w_max: 1.0,
                        w_min: 0.0,
                    },
                    source: match synapse_direction {
                        true => pre_entity,
                        false => post_entity,
                    },
                    target: match synapse_direction {
                        true => post_entity,
                        false => pre_entity,
                    },
                    // weight between 0 and 1
                    weight: rand::thread_rng().gen_range(0.1..=0.3),
                    delay: 1,
                    synapse_type,
                },
                Visibility::Visible,
                GlobalTransform::default(),
                Transform::from_xyz(0.0, 0.0, 0.0),
                // Collider::capsule_y(length / 2.0, 0.05),
            ))
            .with_children(|parent| {
                parent.spawn(PbrBundle {
                    mesh: synapse_mesh.clone(),
                    material: match synapse_type {
                        SynapseType::Excitatory => synapse_material_excitory.clone(),
                        SynapseType::Inhibitory => synapse_material_inhibitory.clone(),
                    },
                    transform: Transform {
                        translation: match synapse_direction {
                            true => synapse_pos_pre,
                            false => synapse_pos_post,
                        },
                        rotation,
                        ..Default::default()
                    },
                    visibility: Visibility::Inherited,
                    ..Default::default()
                });

                parent.spawn(PbrBundle {
                    mesh: synapse_stalk_mesh,
                    material: match synapse_type {
                        SynapseType::Excitatory => synapse_material_excitory.clone(),
                        SynapseType::Inhibitory => synapse_material_inhibitory.clone(),
                    },
                    transform: Transform {
                        translation: midpoint - pre_transform.translation,
                        rotation,
                        ..Default::default()
                    },
                    visibility: Visibility::Inherited,
                    ..Default::default()
                });
            })
            .set_parent(pre_entity)
            .id();

        info!(
            "Synapse created: {:?}, connected {:?} to {:?}",
            synapse, pre_entity, post_entity
        );
    }
}

fn mouse_click(
    windows: Query<&Window>,
    button_inputs: Res<ButtonInput<MouseButton>>,
    query_camera: Query<(&Camera, &GlobalTransform)>,
    rapier_context: Res<RapierContext>,
    ui_state: Res<UiState>,
    egui_settings: Res<bevy_egui::EguiSettings>,
    mut insights: ResMut<Insights>,
) {
    let window = windows.get_single().unwrap();
    if button_inputs.just_pressed(MouseButton::Left) {
        if let Some(cursor_position) = window.cursor_position() {
            let (camera, camera_transform) = query_camera.single();

            // Adjust cursor position to account for the viewport
            let scale_factor = window.scale_factor() * egui_settings.scale_factor;

            let viewport_pos = ui_state.viewport_rect.left_top().to_vec2() * scale_factor;
            let viewport_size = ui_state.viewport_rect.size() * scale_factor;

            let adjusted_cursor_position =
                cursor_position - Vec2::new(viewport_pos.x, viewport_pos.y);

            // Check if the adjusted cursor position is within the viewport bounds
            if adjusted_cursor_position.x >= 0.0
                && adjusted_cursor_position.y >= 0.0
                && adjusted_cursor_position.x <= viewport_size.x
                && adjusted_cursor_position.y <= viewport_size.y
            {
                if let Some(ray) =
                    camera.viewport_to_world(camera_transform, adjusted_cursor_position)
                {
                    // Perform ray casting
                    if let Some((entity, _intersection)) = rapier_context.cast_ray(
                        ray.origin,
                        *ray.direction,
                        f32::MAX,
                        true,
                        QueryFilter::default(),
                    ) {
                        insights.selected_entity = Some(entity);
                        trace!("Clicked on entity: {:?}", entity);
                    } else {
                        insights.selected_entity = None;
                    }
                }
            }
        }
    }
}

fn update_neuron_materials(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut neuron_query: Query<(
        Entity,
        One<&mut dyn NeuronVisualizer>,
        &Handle<StandardMaterial>,
        &ColumnLayer,
    )>,
) {
    for (_entity, neuron, material_handle, layer) in neuron_query.iter_mut() {
        let material = materials.get_mut(material_handle).unwrap();

        material.emissive = layer.get_color_from_activation(neuron.activation_percent());
        material.base_color = layer.get_color();
    }
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true, // HDR is required for bloom
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface, // Using a tonemapper that desaturates to white is recommended for bloom
            transform: Transform::from_xyz(-2.0, 2.5, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        // Enable bloom for the camera
        BloomSettings {
            composite_mode: BloomCompositeMode::Additive,
            high_pass_frequency: 1.0,
            intensity: 0.1,
            low_frequency_boost: 0.8,
            low_frequency_boost_curvature: 1.0,
            prefilter_settings: BloomPrefilterSettings::default(),
        },
        PanOrbitCamera::default(),
        ClusterConfig::Single, // Single cluster for the whole scene as it's small
    ));
}
