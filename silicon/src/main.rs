use std::time::Duration;

use ::neurons::{Neuron, NeuronPlugin};
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
    rapier::crossbeam::epoch::Pointable,
};
use bevy_trait_query::One;
use data::{MembranePlotter, NeuronDataCollectionPlugin};
use neurons::NeuronVisualizer;
use rand::Rng;
use simulator::{time::Clock, SimulationPlugin, SpikeEvent};
use structure::cortical_column::{ColumnLayer, MiniColumn};
use synapses::{simple::SimpleSynapse, SynapsePlugin, SynapseType};
use ui::{state::UiState, SiliconUiPlugin};

mod data;
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
                    filter: "info,silicon=trace".into(),
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
        .add_plugins(NeuronDataCollectionPlugin)
        .add_plugins(SiliconUiPlugin)
        .add_plugins(NeuronPlugin)
        .add_plugins(SimulationPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(SynapsePlugin)
        // .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(Msaa::Sample8)
        .insert_resource(Insights {
            selected_entity: None,
        })
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(5000)))
        .add_systems(FixedUpdate, insert_current)
        .add_systems(PostStartup, notify_setup_done)
        .add_systems(Update, show_select_neuron_synapses)
        .add_systems(
            Update,
            (update_neurons, update_neuron_materials, mouse_click),
        )
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
    mut synapse_query: Query<(&SimpleSynapse, &mut Visibility, &Children)>,
    mut child_query: Query<&mut Visibility, Without<SimpleSynapse>>,
) {
    if let Some(selected_entity) = insights.selected_entity {
        for (synapse, mut visibility, children) in synapse_query.iter_mut() {
            let is_visible = synapse.source == selected_entity || synapse.target == selected_entity;

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

fn insert_current(mut neurons_query: Query<(Entity, One<&mut dyn Neuron>, &ColumnLayer)>) {
    for (entity, mut neuron, layer) in neurons_query.iter_mut() {
        if layer != &ColumnLayer::L4 {
            continue;
        }

        // only insert current into 50% of L4 neurons
        if rand::random::<f64>() < 0.5 {
            trace!("Inserting current into neuron {:?}", entity);
            neuron.add_membrane_potential(rand::thread_rng().gen_range(0.4..=0.8));
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
        let synapse_pos_post = midpoint + (post_transform.translation - midpoint) / 2.0;
        let synapse_pos_pre = midpoint + (pre_transform.translation - midpoint) / 2.0;
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
                SimpleSynapse {
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
                    mesh: synapse_mesh,
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
                        translation: midpoint,
                        rotation,
                        ..Default::default()
                    },
                    visibility: Visibility::Inherited,
                    ..Default::default()
                });
            })
            // .set_parent(parent.get())
            .id();

        info!(
            "Synapse created: {:?}, connected {:?} to {:?}",
            synapse, pre_entity, post_entity
        );
    }
}

fn update_neurons(
    clock: ResMut<Clock>,
    mut neuron_query: Query<(Entity, One<&mut dyn Neuron>, Option<&mut MembranePlotter>)>,
    mut spike_writer: EventWriter<SpikeEvent>,
) {
    for (entity, mut neuron, mut plotter) in neuron_query.iter_mut() {
        let fired = neuron.update(clock.tau);
        if let Some(plotter) = &mut plotter {
            plotter.add_point(neuron.get_membrane_potential(), clock.time);
            if fired {
                plotter.add_spike(clock.time);
            }
        }

        if fired {
            spike_writer.send(SpikeEvent {
                time: clock.time,
                neuron: entity,
            });
        }
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
