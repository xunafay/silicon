use bevy::{
    core::TaskPoolThreadAssignmentPolicy,
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
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
    geometry::Collider,
    pipeline::QueryFilter,
    plugin::{NoUserData, RapierContext, RapierPhysicsPlugin},
};
use data::{MembranePlotter, NeuronDataCollectionPlugin};
use neurons::{
    cortical_column::{ColumnLayer, MiniColumn},
    leaky::LifNeuron,
    synapse::{AllowSynapse, Synapse, SynapseType},
    Clock, IzhikevichNeuron, Neuron, NeuronRuntimePlugin, OscillatingNeuron, Refactory, Spike,
    SpikeEvent, SpikeRecorder,
};
use rand::seq::IteratorRandom;
use ui::{state::UiState, SiliconUiPlugin};
use uom::{
    si::{
        electric_potential::millivolt,
        electrical_resistance::ohm,
        f64::{ElectricPotential, ElectricalResistance, Time as SiTime},
        time::second,
    },
    ConstZero,
};

mod data;
mod neurons;
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
        .add_plugins(NeuronRuntimePlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        // .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(Msaa::Sample8)
        .insert_resource(Insights {
            selected_entity: None,
        })
        .add_systems(
            Update,
            (
                update_neurons::<IzhikevichNeuron>,
                update_neurons::<LifNeuron>,
            ),
        )
        .add_systems(
            Startup,
            (
                (create_neurons, create_synapses::<IzhikevichNeuron>).chain(),
                setup_scene,
            ),
        )
        // .add_systems(PostStartup, hide_meshes) // hide meshes if you need some extra performance
        .add_systems(
            Update,
            (
                update_bloom_settings,
                update_neuron_materials::<LifNeuron>,
                update_neuron_materials::<IzhikevichNeuron>,
                mouse_click,
            ),
        );
    }
}

#[allow(dead_code)]
fn hide_meshes(mut visibilities: Query<&mut Visibility>) {
    for mut visibility in visibilities.iter_mut() {
        *visibility = Visibility::Hidden;
    }
}

fn create_neurons(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let minicolumn = commands
        .spawn((
            MiniColumn,
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
        ))
        .id();

    let mesh = meshes.add(Cuboid::new(0.5, 0.5, 0.5).mesh());

    let mut neurons = vec![];

    for x in -1..1 {
        for y in -1..1 {
            for z in 0..1 {
                let leaky_neuron_material = materials.add(StandardMaterial {
                    emissive: Color::rgb_linear(23000.0, 9000.0, 3000.0),
                    ..Default::default()
                });

                let neuron = commands
                    .spawn((
                        LifNeuron {
                            membrane_potential: -70.0,
                            reset_potential: -90.0,
                            threshold_potential: -55.0,
                            resistance: 1.3,
                            resting_potential: -70.0,
                            refactory_period: 0.09,
                            refactory_counter: 0.0,
                        },
                        Refactory {
                            refractory_period: SiTime::new::<second>(0.09),
                            refactory_counter: SiTime::ZERO,
                        },
                        PbrBundle {
                            mesh: mesh.clone(),
                            material: leaky_neuron_material,
                            visibility: Visibility::Visible,
                            transform: Transform::from_xyz(x as f32, y as f32, z as f32 + -15.0),
                            ..Default::default()
                        },
                        MembranePlotter::new(),
                        Collider::cuboid(0.25, 0.25, 0.25),
                        ColumnLayer::L1,
                        AllowSynapse,
                    ))
                    .set_parent(minicolumn)
                    .id();

                neurons.push(neuron);
            }
        }
    }

    for x in -2..3 {
        for y in -2..3 {
            for z in 0..1 {
                let leaky_neuron_material = materials.add(StandardMaterial {
                    emissive: Color::rgb_linear(23000.0, 9000.0, 3000.0),
                    ..Default::default()
                });

                let neuron = commands
                    .spawn((
                        IzhikevichNeuron {
                            a: 0.1,
                            b: 0.26,
                            c: -60.0,
                            d: 5.0,
                            v: -65.0,
                            u: -14.0,
                        },
                        PbrBundle {
                            mesh: mesh.clone(),
                            material: leaky_neuron_material,
                            visibility: Visibility::Visible,
                            transform: Transform::from_xyz(x as f32, y as f32, z as f32 + -10.0),
                            ..Default::default()
                        },
                        MembranePlotter::new(),
                        Collider::cuboid(0.25, 0.25, 0.25),
                        ColumnLayer::L2,
                        AllowSynapse,
                    ))
                    .set_parent(minicolumn)
                    .id();

                neurons.push(neuron);
            }
        }
    }

    for x in -2..3 {
        for y in -2..3 {
            for z in 0..1 {
                let leaky_neuron_material = materials.add(StandardMaterial {
                    emissive: Color::rgb_linear(23000.0, 9000.0, 3000.0),
                    ..Default::default()
                });

                let neuron = commands
                    .spawn((
                        LifNeuron {
                            membrane_potential: -70.0,
                            reset_potential: -90.0,
                            threshold_potential: -55.0,
                            resistance: 1.3,
                            resting_potential: -70.0,
                            refactory_period: 0.09,
                            refactory_counter: 0.0,
                        },
                        Refactory {
                            refractory_period: SiTime::new::<second>(0.09),
                            refactory_counter: SiTime::ZERO,
                        },
                        PbrBundle {
                            mesh: mesh.clone(),
                            material: leaky_neuron_material,
                            visibility: Visibility::Visible,
                            transform: Transform::from_xyz(x as f32, y as f32, z as f32 + -5.0),
                            ..Default::default()
                        },
                        MembranePlotter::new(),
                        Collider::cuboid(0.25, 0.25, 0.25),
                        ColumnLayer::L3,
                        AllowSynapse,
                    ))
                    .set_parent(minicolumn)
                    .id();

                neurons.push(neuron);
            }
        }
    }

    for x in -2..2 {
        for y in -2..2 {
            for z in 0..1 {
                let oscillating_neuron_material = materials.add(StandardMaterial {
                    emissive: Color::rgb_linear(3000.0, 23000.0, 9000.0),
                    ..Default::default()
                });

                let neuron = commands
                    .spawn((
                        LifNeuron {
                            membrane_potential: -70.0,
                            reset_potential: -90.0,
                            threshold_potential: -55.0,
                            resistance: 1.3,
                            resting_potential: -70.0,
                            refactory_period: 0.09,
                            refactory_counter: 0.0,
                        },
                        OscillatingNeuron {
                            // random frequency between 0.05 and 0.3
                            frequency: 0.01 + (0.2 - 0.01) * rand::random::<f64>(),
                            amplitude: 1.0,
                        },
                        PbrBundle {
                            mesh: mesh.clone(),
                            material: oscillating_neuron_material,
                            transform: Transform::from_xyz(x as f32, y as f32, z as f32),
                            ..Default::default()
                        },
                        MembranePlotter::new(),
                        Collider::cuboid(0.25, 0.25, 0.25),
                        ColumnLayer::L4,
                    ))
                    .set_parent(minicolumn)
                    .id();

                neurons.push(neuron);
            }
        }
    }

    for x in -2..2 {
        for y in -2..2 {
            for z in 0..1 {
                let leaky_neuron_material = materials.add(StandardMaterial {
                    emissive: Color::rgb_linear(23000.0, 9000.0, 3000.0),
                    ..Default::default()
                });

                let neuron = commands
                    .spawn((
                        LifNeuron {
                            membrane_potential: -70.0,
                            reset_potential: -90.0,
                            threshold_potential: -55.0,
                            resistance: 1.3,
                            resting_potential: -70.0,
                            refactory_period: 0.09,
                            refactory_counter: 0.0,
                        },
                        Refactory {
                            refractory_period: SiTime::new::<second>(0.09),
                            refactory_counter: SiTime::ZERO,
                        },
                        PbrBundle {
                            mesh: mesh.clone(),
                            material: leaky_neuron_material,
                            visibility: Visibility::Visible,
                            transform: Transform::from_xyz(x as f32, y as f32, z as f32 + 5.0),
                            ..Default::default()
                        },
                        MembranePlotter::new(),
                        Collider::cuboid(0.25, 0.25, 0.25),
                        ColumnLayer::L5,
                        AllowSynapse,
                    ))
                    .set_parent(minicolumn)
                    .id();

                neurons.push(neuron);
            }
        }
    }

    for x in -1..2 {
        for y in -1..2 {
            for z in 0..1 {
                let leaky_neuron_material = materials.add(StandardMaterial {
                    emissive: Color::rgb_linear(23000.0, 9000.0, 3000.0),
                    ..Default::default()
                });

                let neuron = commands
                    .spawn((
                        LifNeuron {
                            membrane_potential: -70.0,
                            reset_potential: -90.0,
                            threshold_potential: -55.0,
                            resistance: 1.3,
                            resting_potential: -70.0,
                            refactory_period: 0.09,
                            refactory_counter: 0.0,
                        },
                        PbrBundle {
                            mesh: mesh.clone(),
                            material: leaky_neuron_material,
                            visibility: Visibility::Visible,
                            transform: Transform::from_xyz(x as f32, y as f32, z as f32 + 10.0),
                            ..Default::default()
                        },
                        MembranePlotter::new(),
                        Collider::cuboid(0.25, 0.25, 0.25),
                        ColumnLayer::L6,
                        AllowSynapse,
                    ))
                    .set_parent(minicolumn)
                    .id();

                neurons.push(neuron);
            }
        }
    }
}

fn create_synapses<T: Component + Neuron>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    neuron_query: Query<(Entity, &AllowSynapse, &Transform, &Parent)>,
) {
    let synapse_material_excitory = materials.add(StandardMaterial {
        base_color: Color::rgba(0.4, 0.4, 1.0, 0.5),
        emissive: Color::rgb_linear(0.3, 0.3, 200.0), // Bright green emissive color
        alpha_mode: AlphaMode::Blend,                 // Enable blending for translucency
        ..Default::default()
    });

    let synapse_material_inhibitory = materials.add(StandardMaterial {
        base_color: Color::rgba(1.0, 0.4, 0.4, 0.5),
        emissive: Color::rgb_linear(200.0, 0.3, 0.3), // Bright red emissive color
        alpha_mode: AlphaMode::Blend,                 // Enable blending for translucency
        ..Default::default()
    });

    let mut iter = neuron_query.iter_combinations();

    while let Some([(pre_entity, _, pre_transform, parent), (post_entity, _, post_transform, _)]) =
        iter.fetch_next()
    {
        // 20% chance of creating a synapse
        if rand::random::<f64>() < 0.9 {
            continue;
        }
        let midpoint = (pre_transform.translation + post_transform.translation) / 2.0;
        let direction = post_transform.translation - pre_transform.translation;
        let length = direction.length();
        let normalized_direction = direction.normalize();
        let rotation = Quat::from_rotation_arc(Vec3::Y, normalized_direction);
        let synapse_mesh = meshes.add(Capsule3d::new(0.05, length).mesh());

        let synapse_type = if rand::random::<f64>() > 0.2 {
            SynapseType::Excitatory
        } else {
            SynapseType::Inhibitory
        };

        let synapse = commands
            .spawn((
                Synapse {
                    source: pre_entity,
                    target: post_entity,
                    // weight between 0 and 1
                    weight: rand::random::<f64>(),
                    delay: 1,
                    synapse_type: synapse_type,
                },
                PbrBundle {
                    mesh: synapse_mesh,
                    material: match synapse_type {
                        SynapseType::Excitatory => synapse_material_excitory.clone(),
                        SynapseType::Inhibitory => synapse_material_inhibitory.clone(),
                    },
                    transform: Transform {
                        translation: midpoint,
                        rotation,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                // Collider::capsule_y(length / 2.0, 0.05),
            ))
            .set_parent(parent.get())
            .id();

        info!(
            "Synapse created: {:?}, connected {:?} to {:?}",
            synapse, pre_entity, post_entity
        );

        // commands.entity(neuron.clone()).add_child(synapse);
    }
}

fn update_neurons<T: Component + Neuron>(
    clock: ResMut<Clock>,
    mut neuron_query: Query<(
        Entity,
        &mut T,
        Option<&mut MembranePlotter>,
        Option<&mut SpikeRecorder>,
    )>,
    mut spike_writer: EventWriter<SpikeEvent>,
) {
    for (_entity, mut neuron, mut plotter, mut spike_recorder) in neuron_query.iter_mut() {
        let fired = neuron.update(SiTime::new::<second>(clock.tau));

        if let Some(plotter) = &mut plotter {
            plotter.add_point(neuron.get_membrane_potential(), clock.time);
        }

        if fired {
            spike_writer.send(SpikeEvent {
                time: SiTime::new::<second>(clock.time),
                neuron: _entity,
            });

            if let Some(spike_recorder) = &mut spike_recorder {
                spike_recorder.spikes.push(Spike {
                    time: SiTime::new::<second>(clock.time),
                    neuron: _entity,
                });
            }
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
                        println!("Clicked on entity: {:?}", entity);
                    }
                }
            }
        }
    }
}

fn update_neuron_materials<T: Component + Neuron>(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut neuron_query: Query<(Entity, &T, &Handle<StandardMaterial>, &ColumnLayer)>,
) {
    for (_, neuron, material_handle, layer) in neuron_query.iter_mut() {
        let material = materials.get_mut(material_handle).unwrap();
        // if neuron.membrane_potential < leaky.resting_potential {
        //     material.emissive = layer.get_color_from_potential(
        //         leaky.resting_potential.get::<millivolt>() as f32,
        //         leaky.resting_potential.get::<millivolt>() as f32,
        //         neuron.threshold_potential.get::<millivolt>() as f32,
        //     );
        // } else {
        //     material.emissive = layer.get_color_from_potential(
        //         neuron.membrane_potential.get::<millivolt>() as f32,
        //         leaky.resting_potential.get::<millivolt>() as f32,
        //         neuron.threshold_potential.get::<millivolt>() as f32,
        //     );
        // }
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
        BloomSettings::NATURAL,
        PanOrbitCamera::default(),
        ClusterConfig::Single, // Single cluster for the whole scene as it's small
    ));

    // bloom settings text
    commands.spawn(
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        }),
    );
}

fn update_bloom_settings(
    mut camera: Query<(Entity, Option<&mut BloomSettings>), With<Camera>>,
    mut text: Query<&mut Text>,
    mut commands: Commands,
    keycode: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let bloom_settings = camera.single_mut();
    let mut text = text.single_mut();
    let text = &mut text.sections[0].value;

    match bloom_settings {
        (entity, Some(mut bloom_settings)) => {
            *text = "BloomSettings (Toggle: Space)\n".to_string();
            text.push_str(&format!("(Q/A) Intensity: {}\n", bloom_settings.intensity));
            text.push_str(&format!(
                "(W/S) Low-frequency boost: {}\n",
                bloom_settings.low_frequency_boost
            ));
            text.push_str(&format!(
                "(E/D) Low-frequency boost curvature: {}\n",
                bloom_settings.low_frequency_boost_curvature
            ));
            text.push_str(&format!(
                "(R/F) High-pass frequency: {}\n",
                bloom_settings.high_pass_frequency
            ));
            text.push_str(&format!(
                "(T/G) Mode: {}\n",
                match bloom_settings.composite_mode {
                    BloomCompositeMode::EnergyConserving => "Energy-conserving",
                    BloomCompositeMode::Additive => "Additive",
                }
            ));
            text.push_str(&format!(
                "(Y/H) Threshold: {}\n",
                bloom_settings.prefilter_settings.threshold
            ));
            text.push_str(&format!(
                "(U/J) Threshold softness: {}\n",
                bloom_settings.prefilter_settings.threshold_softness
            ));

            if keycode.just_pressed(KeyCode::Space) {
                commands.entity(entity).remove::<BloomSettings>();
            }

            let dt = time.delta_seconds();

            if keycode.pressed(KeyCode::KeyA) {
                bloom_settings.intensity -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::KeyQ) {
                bloom_settings.intensity += dt / 10.0;
            }
            bloom_settings.intensity = bloom_settings.intensity.clamp(0.0, 1.0);

            if keycode.pressed(KeyCode::KeyS) {
                bloom_settings.low_frequency_boost -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::KeyW) {
                bloom_settings.low_frequency_boost += dt / 10.0;
            }
            bloom_settings.low_frequency_boost = bloom_settings.low_frequency_boost.clamp(0.0, 1.0);

            if keycode.pressed(KeyCode::KeyD) {
                bloom_settings.low_frequency_boost_curvature -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::KeyE) {
                bloom_settings.low_frequency_boost_curvature += dt / 10.0;
            }
            bloom_settings.low_frequency_boost_curvature =
                bloom_settings.low_frequency_boost_curvature.clamp(0.0, 1.0);

            if keycode.pressed(KeyCode::KeyF) {
                bloom_settings.high_pass_frequency -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::KeyR) {
                bloom_settings.high_pass_frequency += dt / 10.0;
            }
            bloom_settings.high_pass_frequency = bloom_settings.high_pass_frequency.clamp(0.0, 1.0);

            if keycode.pressed(KeyCode::KeyG) {
                bloom_settings.composite_mode = BloomCompositeMode::Additive;
            }
            if keycode.pressed(KeyCode::KeyT) {
                bloom_settings.composite_mode = BloomCompositeMode::EnergyConserving;
            }

            if keycode.pressed(KeyCode::KeyH) {
                bloom_settings.prefilter_settings.threshold -= dt;
            }
            if keycode.pressed(KeyCode::KeyY) {
                bloom_settings.prefilter_settings.threshold += dt;
            }
            bloom_settings.prefilter_settings.threshold =
                bloom_settings.prefilter_settings.threshold.max(0.0);

            if keycode.pressed(KeyCode::KeyJ) {
                bloom_settings.prefilter_settings.threshold_softness -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::KeyU) {
                bloom_settings.prefilter_settings.threshold_softness += dt / 10.0;
            }
            bloom_settings.prefilter_settings.threshold_softness = bloom_settings
                .prefilter_settings
                .threshold_softness
                .clamp(0.0, 1.0);
        }

        (entity, None) => {
            *text = "Bloom: Off (Toggle: Space)".to_string();

            if keycode.just_pressed(KeyCode::Space) {
                commands.entity(entity).insert(BloomSettings::NATURAL);
            }
        }
    }
}
