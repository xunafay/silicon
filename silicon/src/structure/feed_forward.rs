use analytics::MembranePlotter;
use bevy::{
    asset::Assets,
    hierarchy::{BuildChildren, BuildWorldChildren},
    log::info,
    pbr::{AlphaMode, PbrBundle, StandardMaterial},
    prelude::{Commands, Entity, Mut, ResMut, World},
    render::{
        color::Color,
        mesh::{Mesh, Meshable},
        view::Visibility,
    },
    transform::{
        commands,
        components::{GlobalTransform, Transform},
    },
};
use bevy_math::{
    primitives::{Capsule3d, Cuboid, Cylinder},
    Quat, Vec3,
};
use bevy_rapier3d::geometry::Collider;
use neurons::izhikevich::IzhikevichNeuron;
use rand::Rng;
use simulator::SimpleSpikeRecorder;
use synapses::{
    stdp::{StdpParams, StdpSpikeType, StdpState, StdpSynapse},
    AllowSynapses, SynapseType,
};

use super::layer::ColumnLayer;

pub struct FeedForwardNetwork {
    layers: Vec<Vec<Entity>>,
}

impl FeedForwardNetwork {
    pub fn new() -> Self {
        FeedForwardNetwork { layers: Vec::new() }
    }

    pub fn add_layer(
        &mut self,
        size_x: usize,
        size_y: usize,
        size_z: usize,
        world: &mut World,
        column_layer: Option<ColumnLayer>,
    ) {
        world.resource_scope(|world, mut materials: Mut<Assets<StandardMaterial>>| {
            world.resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                let leaky_neuron_material = materials.add(StandardMaterial {
                    emissive: Color::rgb_linear(23000.0, 9000.0, 3000.0),
                    ..Default::default()
                });
                let mesh = meshes.add(Cuboid::new(0.5, 0.5, 0.5).mesh());

                let mut layer = vec![];

                let column_layer = match column_layer {
                    Some(layer) => layer,
                    None => ColumnLayer::L1,
                };

                for x in 0..size_x {
                    for y in 0..size_y {
                        for z in 0..size_z {
                            let neuron = world
                                .spawn((
                                    IzhikevichNeuron {
                                        v: -70.0,
                                        u: -14.0,
                                        a: 0.02,
                                        b: 0.2,
                                        c: -100.0,
                                        d: 8.0,
                                        synapse_weight_multiplier: 80.0,
                                    },
                                    PbrBundle {
                                        mesh: mesh.clone(),
                                        material: leaky_neuron_material.clone(),
                                        visibility: Visibility::Visible,
                                        transform: Transform::from_xyz(
                                            x as f32,
                                            y as f32,
                                            z as f32 + (self.layers.len() as f32 * -5.0),
                                        ),
                                        ..Default::default()
                                    },
                                    MembranePlotter::new(),
                                    Collider::cuboid(0.25, 0.25, 0.25),
                                    column_layer.clone(),
                                    AllowSynapses,
                                    SimpleSpikeRecorder::default(),
                                ))
                                .id();

                            layer.push(neuron);
                        }
                    }
                }

                self.layers.push(layer);
            });
        });
    }

    fn create_synapse(
        pre_neuron: &Entity,
        post_neuron: &Entity,
        synapse_type: SynapseType,
        weight_range: (f64, f64),
        world: &mut World,
    ) -> Entity {
        let (synapse_material_excitory, synapse_material_inhibitory) =
            world.resource_scope(|world, mut materials: Mut<Assets<StandardMaterial>>| {
                let synapse_material_excitory = materials.add(StandardMaterial {
                    base_color: Color::rgba(0.4, 0.4, 1.0, 0.8),
                    emissive: Color::rgb_linear(0.3, 0.3, 200.0), // Bright green emissive color
                    alpha_mode: AlphaMode::Blend, // Enable blending for translucency
                    ..Default::default()
                });

                let synapse_material_inhibitory = materials.add(StandardMaterial {
                    base_color: Color::rgba(1.0, 0.4, 0.4, 0.8),
                    emissive: Color::rgb_linear(200.0, 0.3, 0.3), // Bright red emissive color
                    alpha_mode: AlphaMode::Blend, // Enable blending for translucency
                    ..Default::default()
                });

                (synapse_material_excitory, synapse_material_inhibitory)
            });

        let pre_transform = world.get::<Transform>(*pre_neuron).unwrap().clone();
        let post_transform = world.get::<Transform>(*post_neuron).unwrap().clone();

        let midpoint = (pre_transform.translation + post_transform.translation) / 2.0;
        let synapse_pos_pre =
            (pre_transform.translation + midpoint) / 2.0 - pre_transform.translation;
        let synapse_pos_post =
            (post_transform.translation + midpoint) / 2.0 - pre_transform.translation;
        let direction = post_transform.translation - pre_transform.translation;
        let length = direction.length();
        let normalized_direction = direction.normalize();
        let rotation = Quat::from_rotation_arc(Vec3::Y, normalized_direction);

        let (synapse_stalk_mesh, synapse_mesh) =
            world.resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                let synapse_stalk_mesh = meshes.add(Capsule3d::new(0.05, length).mesh());
                let synapse_mesh = meshes.add(
                    Cylinder {
                        half_height: 0.2,
                        radius: 0.2,
                    }
                    .mesh(),
                );

                (synapse_stalk_mesh, synapse_mesh)
            });

        let synapse = world
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
                    stdp_state: StdpState {
                        a: 0.0,
                        spike_type: StdpSpikeType::PreSpike,
                    },
                    source: *pre_neuron,
                    target: *post_neuron,
                    // weight between 0 and 1
                    weight: rand::thread_rng().gen_range(weight_range.0..=weight_range.1),
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
                        translation: synapse_pos_post,
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
            .set_parent(*pre_neuron)
            .id();

        info!(
            "Synapse created: {:?}, connected {:?} to {:?}",
            synapse, pre_neuron, post_neuron
        );

        synapse
    }

    pub fn connect_layers(
        &mut self,
        source_layer: usize,
        target_layer: usize,
        connection_chance: f64,
        type_ratio: f64,
        world: &mut World,
    ) {
        if source_layer >= self.layers.len() || target_layer >= self.layers.len() {
            panic!("Invalid layer index");
        }

        for pre_neuron in &self.layers[source_layer] {
            for post_neuron in &self.layers[target_layer] {
                if rand::random::<f64>() > connection_chance {
                    continue;
                }

                let synapse_type = if rand::random::<f64>() < type_ratio {
                    SynapseType::Excitatory
                } else {
                    SynapseType::Inhibitory
                };

                let synapse =
                    Self::create_synapse(pre_neuron, post_neuron, synapse_type, (0.1, 0.3), world);

                info!(
                    "Synapse created: {:?}, connected {:?} to {:?}",
                    synapse, pre_neuron, post_neuron
                );
            }
        }
    }

    pub fn add_wta_layer(
        &mut self,
        size_x: usize,
        size_y: usize,
        size_z: usize,
        world: &mut World,
        colmun_layer: Option<ColumnLayer>,
    ) {
        let (leaky_neuron_material, mesh) =
            world.resource_scope(|world, mut materials: Mut<Assets<StandardMaterial>>| {
                let leaky_neuron_material = materials.add(StandardMaterial {
                    emissive: Color::rgb_linear(23000.0, 9000.0, 3000.0),
                    ..Default::default()
                });

                let mesh = world.resource_scope(|_, mut meshes: Mut<Assets<Mesh>>| {
                    meshes.add(Cuboid::new(0.5, 0.5, 0.5).mesh())
                });

                (leaky_neuron_material, mesh)
            });

        let mut layer = vec![];

        let colmun_layer = match colmun_layer {
            Some(layer) => layer,
            None => ColumnLayer::L1,
        };

        for x in 0..size_x {
            for y in 0..size_y {
                for z in 0..size_z {
                    let neuron = world
                        .spawn((
                            IzhikevichNeuron {
                                v: -70.0,
                                u: -14.0,
                                a: 0.02,
                                b: 0.2,
                                c: -100.0,
                                d: 8.0,
                                synapse_weight_multiplier: 80.0,
                            },
                            PbrBundle {
                                mesh: mesh.clone(),
                                material: leaky_neuron_material.clone(),
                                visibility: Visibility::Visible,
                                transform: Transform::from_xyz(
                                    x as f32,
                                    y as f32,
                                    z as f32 + (self.layers.len() as f32 * -5.0),
                                ),
                                ..Default::default()
                            },
                            MembranePlotter::new(),
                            Collider::cuboid(0.25, 0.25, 0.25),
                            colmun_layer,
                            AllowSynapses,
                            SimpleSpikeRecorder::default(),
                        ))
                        .id();

                    layer.push(neuron);
                }
            }
        }

        // connect every neuron in this layer with each other neuron with an inhibitory synapse

        for pre_neuron in &layer {
            for post_neuron in &layer {
                if pre_neuron == post_neuron {
                    continue;
                }

                Self::create_synapse(
                    pre_neuron,
                    post_neuron,
                    SynapseType::Inhibitory,
                    (2.0, 4.0),
                    world,
                );
            }
        }

        self.layers.push(layer);
    }
}
