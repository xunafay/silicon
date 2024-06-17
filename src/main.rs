use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    log::LogPlugin,
    prelude::*,
};
use cortical_column::CorticalColumn;
use neurons::{Neuron, OscillatingNeuron};
use rand::seq::SliceRandom;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use synapse::Synapse;
mod cortical_column;
mod neurons;
mod synapse;

fn main() {
    App::new().add_plugins(NeuronPlugin).run();
}

pub struct NeuronPlugin;

impl Plugin for NeuronPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MinimalPlugins)
            .add_plugins(LogPlugin {
                level: bevy::log::Level::TRACE,
                filter: "info,silicon=trace".into(),
                ..Default::default()
            })
            .add_systems(Startup, create_neurons)
            .add_systems(Update, update_neurons_system)
            // .add_systems(Startup, create_oscil_neuron)
            // .add_systems(Update, sim_oscil_neurons)
            ;
    }
}

pub struct NeuronRenderPlugin;

impl Plugin for NeuronRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(LogPlugin {
            level: bevy::log::Level::TRACE,
            filter: "info,silicon=trace".into(),
            ..Default::default()
        }))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, update_bloom_settings);
    }
}

fn create_neurons(mut commands: Commands) {
    let cortical_column = commands.spawn(CorticalColumn { x: 0, y: 0 }).id();
    let mut neurons = vec![];

    // create 10 leaky neurons
    for _ in 0..10 {
        let neuron = commands
            .spawn(Neuron {
                membrane_potential: -70.0,
                resting_potential: -70.0,
                reset_potential: -90.0,
                threshold_potential: -55.0,
                resistance: 1.3,
                refractory_period: 0.09,
                refactory_counter: 0.0,
            })
            .set_parent(cortical_column.clone())
            .id();

        neurons.push(neuron);
    }

    // create 5 oscillating neurons
    for _ in 0..5 {
        let neuron = commands
            .spawn(Neuron {
                membrane_potential: -70.0,
                resting_potential: -70.0,
                reset_potential: -90.0,
                threshold_potential: -55.0,
                resistance: 1.3,
                refractory_period: 0.09,
                refactory_counter: 0.0,
            })
            .insert(OscillatingNeuron {
                frequency: 0.32,
                amplitude: 10.0,
            })
            .set_parent(cortical_column.clone())
            .id();

        neurons.push(neuron);
    }

    // create 2.5 synapses for each neuron
    for neuron in neurons.clone() {
        for _ in 0..2 {
            let target_neuron = neurons.choose(&mut rand::thread_rng()).unwrap();
            let synapse = commands
                .spawn(Synapse {
                    source: neuron.clone(),
                    target: target_neuron.clone(),
                    weight: 0.5,
                    delay: 1,
                    synapse_type: synapse::SynapseType::Excitatory,
                })
                .set_parent(neuron)
                .id();

            info!(
                "Synapse created: {:?}, connected {:?} to {:?}",
                synapse, neuron, target_neuron
            );

            commands.entity(neuron.clone()).add_child(synapse);
        }
    }
}

fn update_neurons_system(
    time: Res<Time>,
    mut neuron_query: Query<(Entity, &mut Neuron, Option<&mut OscillatingNeuron>)>,
    synapse_query: Query<(&Parent, &Synapse, Entity)>,
) {
    let mut fired_neurons = vec![];

    for (entity, mut neuron, oscillating) in neuron_query.iter_mut() {
        // Update based on neuron type
        let fired = if let Some(mut oscillating_neuron) = oscillating {
            tick_oscillating(
                &mut neuron,
                &mut oscillating_neuron,
                time.delta_seconds() as f64,
            )
        } else {
            tick_leaky(&mut neuron, time.delta_seconds() as f64)
        };

        if fired {
            trace!("Neuron fired: {:?}", entity);
            fired_neurons.push(entity);
        }
    }

    for neuron in fired_neurons {
        for (parent, synapse, synapse_id) in synapse_query.iter() {
            if parent.get() == neuron {
                let (neuron_id, mut post_synaptic_neuron, _) = neuron_query
                    .get_mut(synapse.target)
                    .expect("Failed to get post synaptic neuron");

                post_synaptic_neuron.membrane_potential += synapse.weight;
                trace!(
                    "Synapse fired: {:?} with target {:?}",
                    synapse_id,
                    neuron_id
                );
            }
        }
    }
}

pub fn tick_leaky(neuron: &mut Neuron, time_step: f64) -> bool {
    if neuron.refactory_counter > 0.0 {
        neuron.refactory_counter -= time_step as f32;
        return false;
    }

    let delta_v =
        neuron.resistance * (neuron.resting_potential - neuron.membrane_potential) * time_step;
    neuron.membrane_potential += delta_v;

    if neuron.membrane_potential >= neuron.threshold_potential {
        neuron.membrane_potential = neuron.reset_potential;
        neuron.refactory_counter = neuron.refractory_period;
        return true;
    }

    false
}

pub fn tick_oscillating(
    neuron: &mut Neuron,
    oscillating: &mut OscillatingNeuron,
    time_step: f64,
) -> bool {
    if neuron.refactory_counter > 0.0 {
        neuron.refactory_counter -= time_step as f32;
        return false;
    }

    let delta_v =
        neuron.resistance * (neuron.resting_potential - neuron.membrane_potential) * time_step;
    let oscillation = oscillating.amplitude
        * (2.0 * std::f64::consts::PI * oscillating.frequency * time_step).sin();
    neuron.membrane_potential += delta_v + oscillation;

    if neuron.membrane_potential >= neuron.threshold_potential {
        neuron.membrane_potential = neuron.reset_potential;
        neuron.refactory_counter = neuron.refractory_period;
        return true;
    }

    false
}

// fire neuron every 3 seconds
fn fire_neuron(mut query: Query<(Entity, &mut Neuron)>, time: Res<Time>) {
    for (entity, mut neuron) in query.iter_mut().skip(1) {
        if time.elapsed_seconds() > 3.0 && time.elapsed_seconds() < 3.1 {
            trace!("Firing neuron {:?}", entity);
            neuron.membrane_potential = neuron.threshold_potential + 1.0;
        }
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true, // 1. HDR is required for bloom
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
            transform: Transform::from_xyz(-2.0, 2.5, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        // 3. Enable bloom for the camera
        BloomSettings::NATURAL,
    ));

    let material_emissive1 = materials.add(StandardMaterial {
        emissive: Color::rgb_linear(23000.0, 9000.0, 3000.0), // 4. Put something bright in a dark environment to see the effect
        ..default()
    });
    let material_emissive2 = materials.add(StandardMaterial {
        emissive: Color::rgb_linear(3000.0, 23000.0, 9000.0),
        ..default()
    });
    let material_emissive3 = materials.add(StandardMaterial {
        emissive: Color::rgb_linear(9000.0, 3000.0, 23000.0),
        ..default()
    });
    let material_non_emissive = materials.add(StandardMaterial {
        base_color: Color::GRAY,
        ..default()
    });

    let mesh = meshes.add(Cuboid::new(0.5, 0.5, 0.5).mesh());

    for x in -5..5 {
        for y in -5..5 {
            // This generates a pseudo-random integer between `[0, 6)`, but deterministically so
            // the same spheres are always the same colors.
            let mut hasher = DefaultHasher::new();
            (x, y).hash(&mut hasher);
            let rand = (hasher.finish() - 2) % 3;

            let material = match rand {
                0 => material_emissive1.clone(),
                1 => material_emissive2.clone(),
                2 => material_emissive3.clone(),
                3..=5 => material_non_emissive.clone(),
                _ => unreachable!(),
            };

            commands.spawn(PbrBundle {
                mesh: mesh.clone(),
                material,
                transform: Transform::from_xyz(x as f32, y as f32, 0.0),
                ..default()
            });
        }
    }

    // example instructions
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
