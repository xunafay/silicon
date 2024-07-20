#![allow(clippy::type_complexity)]

use std::{ops::Deref, time::Duration};

use bevy::{
    core::TaskPoolThreadAssignmentPolicy,
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings},
        tonemapping::Tonemapping,
    },
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
use silicon_core::{Clock, Neuron, NeuronVisualizer, SpikeRecorder, ValueRecorderConfig};
use simulator::SimulationPlugin;
use structure::{feed_forward::FeedForwardNetwork, layer::ColumnLayer};
use synapses::{
    simple::SimpleSynapse, stdp::StdpSynapse, DeferredStdpEvent, Synapse, SynapsePlugin,
};
use transcoder::{nlp::string_to_spike_train, population::PopulationEncoder};
use ui::{
    state::{PlotterConfig, UiState},
    SiliconUiPlugin,
};

mod structure;
mod ui;

fn main() {
    App::new().add_plugins(SiliconPlugin).run();
}

#[derive(Resource)]
pub struct Insights {
    pub selected_entity: Option<Entity>,
}

#[derive(Debug, Clone, Reflect, Resource, PartialEq)]
pub enum Class {
    Hello,
    World,
}

pub struct SiliconPlugin;

impl Plugin for SiliconPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    level: bevy::log::Level::TRACE,
                    filter: "info,silicon=trace,simulator=trace,synapses=trace".into(),
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
        .add_plugins((
            SimulationPlugin,
            NeuronPlugin,
            SynapsePlugin,
            SiliconUiPlugin,
        ))
        // .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(Msaa::Sample8)
        .insert_resource(Insights {
            selected_entity: None,
        })
        // .insert_resource(SynapseDecay {
        //     interval: 1.0,
        //     amount: 0.0001,
        //     next_decay: 1.0,
        // })
        .insert_resource(ValueRecorderConfig { window_size: 10000 })
        .insert_resource(PlotterConfig { window_size: 300 })
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(5000)))
        .insert_resource(EncoderState::default())
        .add_systems(Startup, (create_neurons, setup_scene))
        .add_systems(PostStartup, notify_setup_done)
        .add_systems(
            Update,
            (
                insert_current,
                show_select_neuron_synapses,
                update_neuron_materials,
                mouse_click,
            ),
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
    pub next_presentation_time: f64,
    pub time_between_classes: f64,
    pub current_class: Class,
    pub encoders: Vec<(Class, PopulationEncoder)>,
}

impl Default for EncoderState {
    fn default() -> Self {
        EncoderState {
            current_class: Class::Hello,
            encoders: vec![],
            time_between_classes: 5.0,
            next_presentation_time: 5.0,
        }
    }
}

fn reward(firing_rate: i32, target_rate: i32) -> f64 {
    let target_rate = match target_rate {
        0 => 0.001,
        _ => target_rate as f64,
    };

    let error = error(firing_rate as f64, target_rate);
    let reward = 1.0 - error / target_rate;
    reward.clamp(-1.0, 1.0)
}

fn error(firing_rate: f64, target_rate: f64) -> f64 {
    let error = (target_rate - firing_rate).abs();
    error
}

fn insert_current(
    mut neurons_query: Query<(
        Entity,
        One<&mut dyn Neuron>,
        &ColumnLayer,
        One<&dyn SpikeRecorder>,
    )>,
    clock: Res<Clock>,
    mut encoder: ResMut<EncoderState>,
    mut deferred_stdp_events: ResMut<Events<DeferredStdpEvent>>,
    mut stdp_synapses: Query<(Entity, &mut StdpSynapse)>,
) {
    if clock.time < encoder.next_presentation_time {
        return;
    }

    // presentation time is done, calculate reward for the current class
    // apply reward modulated STDP
    // present the next class

    // == calculate reward ==

    let mut output_neurons = neurons_query
        .iter()
        .filter(|(_, _, layer, _)| *layer == &ColumnLayer::L6)
        .collect::<Vec<_>>();
    output_neurons.sort_by(|(a, _, _, _), (b, _, _, _)| {
        let a = a.generation() as f64 + (a.index() as f64 / 10.0);
        let b = b.generation() as f64 + (b.index() as f64 / 10.0);
        a.partial_cmp(&b).unwrap()
    });

    let mut class_for_neuron = Class::Hello;
    let mut correct_class_spikes = 0;
    let mut wrong_class_spikes = 0;

    for (entity, _, _, spike_recorder) in output_neurons {
        trace!(
            "Calculating reward for neuron {:?} with class {:?}",
            entity,
            class_for_neuron
        );
        let spikes = spike_recorder
            .get_spikes()
            .iter()
            .filter(|s| **s >= clock.time - encoder.time_between_classes)
            .count();

        if class_for_neuron == encoder.current_class {
            correct_class_spikes += spikes as i32;
        } else {
            wrong_class_spikes += spikes as i32;
        }

        class_for_neuron = match class_for_neuron {
            Class::Hello => Class::World,
            Class::World => Class::Hello,
        };
    }

    trace!(
        "Correct class spikes: {}\t Wrong class spikes: {}\t expected class: {:?}",
        correct_class_spikes,
        wrong_class_spikes,
        encoder.current_class
    );

    let correct_error = error(correct_class_spikes as f64, 3.0);
    let wrong_error = error(wrong_class_spikes as f64, 0.0);

    let mut reward = match correct_error > wrong_error {
        true => reward(correct_class_spikes, 3),
        false => reward(wrong_class_spikes, 3),
    };

    trace!("Reward: {}", reward);

    if reward == 0.0 {
        trace!("reward is zero, randomizing it for network exploration purposes");
        reward = rand::thread_rng().gen_range(-2.0..=2.0);
        trace!("Randomized reward: {}", reward);
    }

    // == apply reward modulated STDP ==
    for event in deferred_stdp_events.drain() {
        let synapse = stdp_synapses
            .iter_mut()
            .find(|(entity, _)| *entity == event.synapse);

        if let Some((_, mut synapse)) = synapse {
            trace!("applying stdp to {:?} with\ndelta weight {}\nreward modulated delta weight: {}\nnew weight {}",
                event.synapse,
                event.delta_weight,
                event.delta_weight * reward,
                synapse.weight + event.delta_weight
            );

            synapse.weight += event.delta_weight * reward;
            synapse.weight = synapse
                .weight
                .clamp(synapse.stdp_params.w_min, synapse.stdp_params.w_max);
        }
    }

    // == present the next class ==
    encoder.next_presentation_time = clock.time + encoder.time_between_classes;

    encoder.current_class = match encoder.current_class {
        Class::Hello => Class::World,
        Class::World => Class::Hello,
    };

    let encoder = encoder
        .encoders
        .iter()
        .find(|(class, _)| *class == encoder.current_class);

    if let Some((_, encoder)) = encoder {
        let population = encoder.neurons.clone();
        let neurons = neurons_query
            .iter_mut()
            .filter(|(entity, _, _, _)| population.contains(entity))
            .collect::<Vec<_>>();

        for (_, mut neuron, _, _) in neurons {
            neuron.insert_current(rand::thread_rng().gen_range(1.6..=1.8));
        }
    }
}

fn create_neurons(world: &mut World) {
    // MiniColumn::create(commands, meshes, materials);

    let mut ffn = FeedForwardNetwork::new();
    ffn.add_layer(3, 3, 1, world, Some(ColumnLayer::L1));
    ffn.add_layer(3, 3, 1, world, Some(ColumnLayer::L2));
    // ffn.add_layer(3, 3, 1, world, Some(ColumnLayer::L3));
    ffn.add_layer(3, 3, 1, world, Some(ColumnLayer::L4));
    // ffn.add_layer(3, 3, 1, world, Some(ColumnLayer::L5));
    ffn.add_wta_layer(2, 1, 1, world, Some(ColumnLayer::L6));
    ffn.connect_layers(0, 1, 0.8, 0.8, world);
    ffn.connect_layers(1, 2, 0.8, 0.8, world);
    ffn.connect_layers(2, 3, 1.0, 0.8, world);

    ffn.connect_layers(1, 0, 0.2, 0.8, world);
    ffn.connect_layers(2, 1, 0.2, 0.8, world);
    ffn.connect_layers(3, 2, 0.8, 0.8, world);
    // ffn.connect_layers(3, 4, 0.8, 0.8, world);
    // ffn.connect_layers(4, 5, 1.0, 0.8, world);

    world.resource_scope(|world, mut encoder: Mut<EncoderState>| {
        let neurons = world
            .query::<(Entity, &mut dyn Neuron, &ColumnLayer)>()
            .iter(world)
            .filter(|(_, _, layer)| *layer == &ColumnLayer::L1)
            .map(|(entity, _, _)| entity)
            .collect::<Vec<_>>();

        encoder.encoders.push((
            Class::Hello,
            PopulationEncoder::from_sample_rate(&neurons, 0.5),
        ));

        encoder.encoders.push((
            Class::World,
            PopulationEncoder::from_sample_rate(&neurons, 0.5),
        ));
    });
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
