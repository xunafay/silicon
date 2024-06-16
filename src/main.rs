use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        core_2d::Core2dPlugin,
        tonemapping::Tonemapping,
    },
    prelude::*,
    time::TimePlugin,
};
use neuron::Neuron;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use synapse::Synapse;

mod neuron;
mod synapse;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (create_neurons))
        .add_systems(Update, (simulate_neurons, fire_neuron))
        .run();
}

fn create_neurons(mut commands: Commands) {
    let neuron_1 = neuron::Neuron::new();
    let neuron_2 = neuron::Neuron::new();
    let neuron_1 = commands.spawn(neuron_1).id();
    let neuron_2 = commands.spawn(neuron_2).id();

    let synapse_1 = commands
        .spawn(synapse::Synapse::new(neuron_1, neuron_2))
        .id();

    commands.entity(neuron_1).add_child(synapse_1);
}

fn simulate_neurons(
    mut neurons: Query<(Entity, &mut Neuron)>,
    synapses: Query<&Synapse>,
    time: Res<Time>,
) {
    println!("delta_seconds_f64: {}", time.delta_seconds_f64()); // Collect all neurons that need to be processed
    let mut active_neurons = vec![];

    for (entity, mut neuron) in neurons.iter_mut() {
        if neuron.tick(time.delta_seconds_f64()) {
            println!("Neuron fired: {:?}", entity);
            active_neurons.push(entity);
        }
    }

    // Apply synapse effects for each active neuron
    for neuron_entity in active_neurons {
        for synapse in synapses.iter() {
            if synapse.pre_synaptic_neuron == neuron_entity {
                if let Ok(post_synaptic_neuron) = neurons.get_mut(synapse.post_synaptic_neuron) {
                    let (_, mut neuron) = post_synaptic_neuron;

                    neuron.apply_synapse(synapse);
                }
            }
        }
    }
}

// fire neuron every 3 seconds
fn fire_neuron(mut query: Query<(Entity, &mut Neuron)>, time: Res<Time>) {
    for (entity, mut neuron) in query.iter_mut().skip(1) {
        if time.elapsed_seconds() > 3.0 && time.elapsed_seconds() < 3.1 {
            println!("Firing neuron {:?}", entity);
            neuron.set_membrane_potential(30.0);
        }
    }
}

fn setup_plot(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // commands.spawn(Camera2dBundle::default());
    // let mut plot = Plot::default();
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

            commands.spawn((
                PbrBundle {
                    mesh: mesh.clone(),
                    material,
                    transform: Transform::from_xyz(x as f32, y as f32, 0.0),
                    ..default()
                },
                Bouncing,
            ));
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

// ------------------------------------------------------------------------------------------------

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

#[derive(Component)]
struct Bouncing;

fn bounce_spheres(time: Res<Time>, mut query: Query<&mut Transform, With<Bouncing>>) {
    for mut transform in query.iter_mut() {
        transform.translation.y =
            (transform.translation.x + transform.translation.z + time.elapsed_seconds()).sin();
    }
}
