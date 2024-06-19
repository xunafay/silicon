use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    log::LogPlugin,
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::{
    egui::{self},
    EguiContext, EguiContexts, EguiPlugin,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use cortical_column::CorticalColumn;
use data::MembranePlotter;
use egui_plot::{Legend, Line, Plot};
use neurons::{
    LeakyNeuron, Neuron, OscillatingNeuron, Refactory, Spike, SpikeEvent, SpikeRecorder,
};
use rand::seq::IteratorRandom;
use synapse::Synapse;
use uom::{
    si::{
        electric_potential::millivolt,
        electrical_resistance::ohm,
        f64::{ElectricPotential, ElectricalResistance, Time as SiTime},
        time::second,
    },
    ConstZero,
};
mod cortical_column;
mod data;
mod neurons;
mod synapse;

fn main() {
    App::new().add_plugins(NeuronRenderPlugin).run();
}

#[derive(Resource)]
pub struct Clock {
    pub time: f64,
    pub tau: f64,
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
            // .add_systems(Update, update_neurons_system)
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
        .add_plugins(EguiPlugin)
        .add_plugins(bevy_inspector_egui::DefaultInspectorConfigPlugin) // adds default options and `InspectorEguiImpl`s
        .add_plugins(PanOrbitCameraPlugin)
        .insert_resource(Msaa::Sample8)
        .add_systems(
            Startup,
            ((create_neurons, create_synapses).chain(), setup_scene),
        )
        // .add_systems(PostStartup, hide_meshes) // hide meshes if you need some extra performance
        .add_systems(
            Update,
            (
                update_leaky_neurons,
                update_oscillating_neurons,
                update_bloom_settings,
                update_synapses,
                update_materials,
                ui_example_system,
                inspector_ui,
                update_plotters,
            ),
        )
        .add_systems(PostUpdate, update_clock)
        .add_event::<SpikeEvent>();
    }
}

fn ui_example_system(
    clock: Res<Clock>,
    mut contexts: EguiContexts,
    plotters: Query<(Entity, &MembranePlotter)>,
) {
    let (entity, first_plotter) = plotters.iter().next().unwrap();
    egui::Window::new("Info").show(contexts.ctx_mut(), |ui| {
        ui.label(format!("Time: {:.2}", clock.time));
        let plot = Plot::new("Test").legend(Legend::default());
        plot.show(ui, |plot_ui| {
            plot_ui.line(Line::new(first_plotter.plot_points()).name(format!("{:?}", entity)));
        });
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

    egui::Window::new("UI").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // equivalent to `WorldInspectorPlugin`
            bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);

            egui::CollapsingHeader::new("Materials").show(ui, |ui| {
                bevy_inspector_egui::bevy_inspector::ui_for_assets::<StandardMaterial>(world, ui);
            });

            ui.heading("Entities");
            bevy_inspector_egui::bevy_inspector::ui_for_world_entities(world, ui);
        });
    });
}

fn update_clock(mut clock: ResMut<Clock>) {
    clock.time += clock.tau;
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
    commands.spawn((
        CorticalColumn { x: 0, y: 0 },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    commands.insert_resource(Clock {
        time: 0.0,
        tau: 0.1,
    });

    let mesh = meshes.add(Cuboid::new(0.5, 0.5, 0.5).mesh());

    let mut neurons = vec![];

    for x in -5..5 {
        for y in -5..5 {
            let leaky_neuron_material = materials.add(StandardMaterial {
                emissive: Color::rgb_linear(23000.0, 9000.0, 3000.0),
                ..Default::default()
            });

            let neuron = commands
                .spawn((
                    Neuron {
                        membrane_potential: ElectricPotential::new::<millivolt>(-70.0),
                        reset_potential: ElectricPotential::new::<millivolt>(-90.0),
                        threshold_potential: ElectricPotential::new::<millivolt>(-55.0),
                        resistance: ElectricalResistance::new::<ohm>(1.3),
                    },
                    LeakyNeuron {
                        resting_potential: ElectricPotential::new::<millivolt>(-70.0),
                    },
                    Refactory {
                        refractory_period: SiTime::new::<second>(0.09),
                        refactory_counter: SiTime::ZERO,
                    },
                    PbrBundle {
                        mesh: mesh.clone(),
                        material: leaky_neuron_material,
                        visibility: Visibility::Visible,
                        transform: Transform::from_xyz(x as f32, y as f32, -10.0),
                        ..Default::default()
                    },
                    MembranePlotter::new(),
                ))
                // .set_parent(cortical_column.clone())
                .id();

            neurons.push(neuron);
        }
    }

    for x in -2..2 {
        for y in -2..2 {
            let oscillating_neuron_material = materials.add(StandardMaterial {
                emissive: Color::rgb_linear(3000.0, 23000.0, 9000.0),
                ..Default::default()
            });

            let neuron = commands
                .spawn((
                    Neuron {
                        membrane_potential: ElectricPotential::new::<millivolt>(-70.0),
                        reset_potential: ElectricPotential::new::<millivolt>(-90.0),
                        threshold_potential: ElectricPotential::new::<millivolt>(-55.0),
                        resistance: ElectricalResistance::new::<ohm>(1.3),
                    },
                    LeakyNeuron {
                        resting_potential: ElectricPotential::new::<millivolt>(-70.0),
                    },
                    OscillatingNeuron {
                        // random frequency between 0.05 and 0.3
                        frequency: 0.01 + (0.2 - 0.01) * rand::random::<f64>(),
                        amplitude: 1.0,
                    },
                    PbrBundle {
                        mesh: mesh.clone(),
                        material: oscillating_neuron_material,
                        transform: Transform::from_xyz(x as f32, y as f32, 10.0),
                        ..Default::default()
                    },
                ))
                // .set_parent(cortical_column.clone())
                .id();

            neurons.push(neuron);
        }
    }
}

fn update_plotters(mut plotter_query: Query<(&Neuron, &mut MembranePlotter)>, clock: Res<Clock>) {
    for (neuron, mut membrane_plotter) in plotter_query.iter_mut() {
        membrane_plotter.add_point(
            neuron.membrane_potential.get::<millivolt>(),
            SiTime::new::<second>(clock.time).get::<second>(),
        );
    }
}

fn create_synapses(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut oscillating_neuron_query: Query<(Entity, &mut Neuron, &Transform, &OscillatingNeuron)>,
    leaky_neuron_query: Query<
        (Entity, &Neuron, &Transform, &LeakyNeuron),
        Without<OscillatingNeuron>,
    >,
) {
    let synapse_material = materials.add(StandardMaterial {
        base_color: Color::rgba(0.0, 1.0, 0.0, 0.5), // Green with 50% opacity
        emissive: Color::rgb_linear(0.2, 200.0, 0.2), // Bright green emissive color
        alpha_mode: AlphaMode::Blend,                // Enable blending for translucency
        ..Default::default()
    });

    for (pre_entity, _pre_neuron, pre_transform, _) in oscillating_neuron_query.iter_mut() {
        for _ in 0..4 {
            let (post_entity, _post_neuron, post_transform, _) = leaky_neuron_query
                .iter()
                .choose(&mut rand::thread_rng())
                .unwrap();

            let midpoint = (pre_transform.translation + post_transform.translation) / 2.0;
            let direction = post_transform.translation - pre_transform.translation;
            let length = direction.length();
            let rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());
            let synapse_mesh = meshes.add(Capsule3d::new(0.05, length).mesh());

            let synapse = commands
                .spawn((
                    Synapse {
                        source: pre_entity.clone(),
                        target: post_entity.clone(),
                        // weight between 0 and 1
                        weight: rand::random::<f64>(),
                        delay: 1,
                        synapse_type: synapse::SynapseType::Excitatory,
                    },
                    PbrBundle {
                        mesh: synapse_mesh,
                        material: synapse_material.clone(),
                        transform: Transform {
                            translation: midpoint,
                            rotation,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                ))
                // .set_parent(neuron)
                .id();

            info!(
                "Synapse created: {:?}, connected {:?} to {:?}",
                synapse, pre_entity, post_entity
            );

            // commands.entity(neuron.clone()).add_child(synapse);
        }
    }
}

fn update_leaky_neurons(
    clock: ResMut<Clock>,
    mut neuron_query: Query<(
        Entity,
        &mut Neuron,
        &mut LeakyNeuron,
        &mut Refactory,
        Option<&mut SpikeRecorder>,
    )>,
    mut spike_writer: EventWriter<SpikeEvent>,
) {
    for (entity, mut neuron, leaky, mut refactory, spike_recorder) in neuron_query.iter_mut() {
        if refactory.refactory_counter > SiTime::ZERO {
            refactory.refactory_counter -= SiTime::new::<second>(0.1);
            continue;
        }

        let delta_v = (leaky.resting_potential.get::<millivolt>()
            - neuron.membrane_potential.get::<millivolt>())
            * clock.tau;

        // trace!("Leaky neuron({:?}) delta_v: {:?}", entity, delta_v);
        // trace!(
        //     "Leaky neuron({:?}) membrane_potential: {:?}",
        //     entity,
        //     neuron.membrane_potential.get::<millivolt>()
        // );
        neuron.membrane_potential += ElectricPotential::new::<millivolt>(delta_v);

        if neuron.membrane_potential > neuron.threshold_potential {
            neuron.membrane_potential = neuron.reset_potential;
            refactory.refactory_counter = refactory.refractory_period;

            // trace!("Leaky neuron fired: {:?}", entity);
            spike_writer.send(SpikeEvent {
                time: SiTime::new::<second>(clock.time),
                neuron: entity,
            });

            if let Some(mut spike_recorder) = spike_recorder {
                spike_recorder.spikes.push(Spike {
                    time: SiTime::new::<second>(clock.time),
                    neuron: entity,
                });
            }
        }
    }
}

fn update_oscillating_neurons(
    clock: ResMut<Clock>,
    mut neuron_query: Query<(
        Entity,
        &mut Neuron,
        &mut LeakyNeuron,
        &mut OscillatingNeuron,
        Option<&mut SpikeRecorder>,
    )>,
    mut spike_writer: EventWriter<SpikeEvent>,
) {
    for (entity, mut neuron, _, oscillating, spike_recorder) in neuron_query.iter_mut() {
        let delta_v = (neuron.resistance.get::<ohm>()
            * (neuron.threshold_potential.get::<millivolt>() + 5.0
                - neuron.membrane_potential.get::<millivolt>()))
            * clock.tau
            * oscillating.frequency;

        neuron.membrane_potential += ElectricPotential::new::<millivolt>(delta_v);

        if neuron.membrane_potential >= neuron.threshold_potential {
            neuron.membrane_potential = neuron.reset_potential;

            if let Some(mut spike_recorder) = spike_recorder {
                spike_recorder.spikes.push(Spike {
                    time: SiTime::new::<second>(clock.time),
                    neuron: entity,
                });
            }

            spike_writer.send(SpikeEvent {
                time: SiTime::new::<second>(clock.time),
                neuron: entity,
            });
        }
    }
}

fn update_synapses(
    mut synapse_query: Query<&Synapse>,
    mut spike_reader: EventReader<SpikeEvent>,
    mut neuron_query: Query<(Entity, &mut Neuron, &mut LeakyNeuron)>,
) {
    // return;
    for spike_event in spike_reader.read() {
        for synapse in synapse_query.iter_mut() {
            if synapse.source == spike_event.neuron {
                let (_, mut target_neuron, leaky) = neuron_query.get_mut(synapse.target).unwrap();
                let threshold_potential = target_neuron.threshold_potential.get::<millivolt>();
                let resting_potential = leaky.resting_potential.get::<millivolt>();

                let delta_v = synapse.weight * (threshold_potential - resting_potential);
                // trace!("Synapse fired: {:?}, delta_v: {:?}", synapse, delta_v);
                target_neuron.membrane_potential += ElectricPotential::new::<millivolt>(delta_v);
            }
        }
    }
}

fn update_materials(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut neuron_query: Query<(Entity, &Neuron, &LeakyNeuron, &Handle<StandardMaterial>)>,
) {
    for (_, neuron, leaky, material_handle) in neuron_query.iter_mut() {
        let material = materials.get_mut(material_handle).unwrap();
        if neuron.membrane_potential < leaky.resting_potential {
            material.emissive = Color::rgb_linear(23000.0, 9000.0, 3000.0);
        } else {
            material.emissive = membrane_potential_to_emissive(
                neuron.membrane_potential.get::<millivolt>() as f32,
                leaky.resting_potential.get::<millivolt>() as f32,
                neuron.threshold_potential.get::<millivolt>() as f32,
            );
        }
    }
}

// ranges from Color::rgb_linear(23000.0, 9000.0, 3000.0) to Color::rgb_linear(0.0, 0.0, 0.0) based on
// membrane potential compared to resting potential
fn membrane_potential_to_emissive(
    membrane_potential: f32,
    resting_potential: f32,
    threshold_potential: f32,
) -> Color {
    Color::rgb_linear(
        refit_to_range(
            membrane_potential,
            resting_potential,
            threshold_potential,
            0.0,
            23000.0,
        ),
        refit_to_range(
            membrane_potential,
            resting_potential,
            threshold_potential,
            0.0,
            9000.0,
        ),
        refit_to_range(
            membrane_potential,
            resting_potential,
            threshold_potential,
            0.0,
            3000.0,
        ),
    )
}

fn refit_to_range(n: f32, start1: f32, stop1: f32, start2: f32, stop2: f32) -> f32 {
    ((n - start1) / (stop1 - start1)) * (stop2 - start2) + start2
}

fn setup_scene(mut commands: Commands) {
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
        PanOrbitCamera::default(),
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
