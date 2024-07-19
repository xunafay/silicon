use analytics::MembranePlotter;
use bevy::{
    asset::Assets,
    color::{Color, LinearRgba},
    hierarchy::BuildChildren,
    pbr::{PbrBundle, StandardMaterial},
    prelude::{Bundle, Commands, Component, ResMut},
    render::{
        mesh::{Mesh, Meshable},
        view::Visibility,
    },
    transform::components::{GlobalTransform, Transform},
};
use bevy_math::primitives::Cuboid;
use bevy_rapier3d::geometry::Collider;
use neurons::izhikevich::IzhikevichNeuron;
use simulator::SimpleSpikeRecorder;
use synapses::AllowSynapses;

use super::layer::ColumnLayer;

#[derive(Component, Debug)]
pub struct MacroColumn;

#[derive(Component, Debug)]
pub struct MiniColumn;

#[derive(Bundle, Debug)]
struct MiniColumnBundle {
    mini_column: MiniColumn,
    layer: ColumnLayer,
}

impl MiniColumn {
    pub fn create(
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
                        emissive: LinearRgba::rgb(23.0, 9.0, 3.0),
                        ..Default::default()
                    });

                    let neuron = commands
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
                                material: leaky_neuron_material,
                                visibility: Visibility::Visible,
                                transform: Transform::from_xyz(
                                    x as f32,
                                    y as f32,
                                    z as f32 + -15.0,
                                ),
                                ..Default::default()
                            },
                            MembranePlotter::new(),
                            Collider::cuboid(0.25, 0.25, 0.25),
                            ColumnLayer::L1,
                            AllowSynapses,
                            SimpleSpikeRecorder::default(),
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
                        emissive: LinearRgba::rgb(23.0, 9.0, 3.0),
                        ..Default::default()
                    });

                    let neuron = commands
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
                                material: leaky_neuron_material,
                                visibility: Visibility::Visible,
                                transform: Transform::from_xyz(
                                    x as f32,
                                    y as f32,
                                    z as f32 + -10.0,
                                ),
                                ..Default::default()
                            },
                            MembranePlotter::new(),
                            Collider::cuboid(0.25, 0.25, 0.25),
                            ColumnLayer::L2,
                            SimpleSpikeRecorder::default(),
                            AllowSynapses,
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
                        emissive: LinearRgba::rgb(23.0, 9.0, 3.0),
                        ..Default::default()
                    });

                    let neuron = commands
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
                                material: leaky_neuron_material,
                                visibility: Visibility::Visible,
                                transform: Transform::from_xyz(x as f32, y as f32, z as f32 + -5.0),
                                ..Default::default()
                            },
                            MembranePlotter::new(),
                            Collider::cuboid(0.25, 0.25, 0.25),
                            SimpleSpikeRecorder::default(),
                            ColumnLayer::L3,
                            AllowSynapses,
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
                        emissive: LinearRgba::rgb(23.0, 9.0, 3.0),
                        ..Default::default()
                    });

                    let neuron = commands
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
                                material: oscillating_neuron_material,
                                transform: Transform::from_xyz(x as f32, y as f32, z as f32),
                                ..Default::default()
                            },
                            MembranePlotter::new(),
                            Collider::cuboid(0.25, 0.25, 0.25),
                            SimpleSpikeRecorder::default(),
                            ColumnLayer::L4,
                            AllowSynapses,
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
                        emissive: LinearRgba::rgb(23.0, 9.0, 3.0),
                        ..Default::default()
                    });

                    let neuron = commands
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
                                material: leaky_neuron_material,
                                visibility: Visibility::Visible,
                                transform: Transform::from_xyz(x as f32, y as f32, z as f32 + 5.0),
                                ..Default::default()
                            },
                            MembranePlotter::new(),
                            Collider::cuboid(0.25, 0.25, 0.25),
                            ColumnLayer::L5,
                            SimpleSpikeRecorder::default(),
                            AllowSynapses,
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
                        emissive: LinearRgba::rgb(23.0, 9.0, 3.0),
                        ..Default::default()
                    });

                    let neuron = commands
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
                                material: leaky_neuron_material,
                                visibility: Visibility::Visible,
                                transform: Transform::from_xyz(x as f32, y as f32, z as f32 + 10.0),
                                ..Default::default()
                            },
                            MembranePlotter::new(),
                            Collider::cuboid(0.25, 0.25, 0.25),
                            ColumnLayer::L6,
                            SimpleSpikeRecorder::default(),
                            AllowSynapses,
                        ))
                        .set_parent(minicolumn)
                        .id();

                    neurons.push(neuron);
                }
            }
        }
    }
}
