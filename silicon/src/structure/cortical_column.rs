use analytics::MembranePlotter;
use bevy::{
    asset::Assets,
    hierarchy::BuildChildren,
    pbr::{PbrBundle, StandardMaterial},
    prelude::{Bundle, Commands, Component, ResMut},
    render::{
        color::Color,
        mesh::{Mesh, Meshable},
        view::Visibility,
    },
    transform::components::{GlobalTransform, Transform},
};
use bevy_math::primitives::Cuboid;
use bevy_rapier3d::geometry::Collider;
use neurons::izhikevich::IzhikevichNeuron;
use silicon_core::SpikeRecorder;
use simulator::SimpleSpikeRecorder;
use synapses::AllowSynapses;

#[derive(Component, Debug)]
pub struct MacroColumn;

#[derive(Component, Debug)]
pub struct MiniColumn;

#[derive(Component, Debug, PartialEq)]
pub enum ColumnLayer {
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
}

impl ColumnLayer {
    pub fn get_color(&self) -> Color {
        match self {
            ColumnLayer::L1 => Color::rgb(0.0, 0.0, 1.0),
            ColumnLayer::L2 => Color::rgb(0.0, 0.5, 1.0),
            ColumnLayer::L3 => Color::rgb(0.0, 1.0, 1.0),
            ColumnLayer::L4 => Color::rgb(0.5, 1.0, 0.5),
            ColumnLayer::L5 => Color::rgb(1.0, 1.0, 0.0),
            ColumnLayer::L6 => Color::rgb(1.0, 0.5, 0.0),
        }
    }

    pub fn get_color_from_potential(
        &self,
        membrane_potential: f32,
        resting_potential: f32,
        threshold_potential: f32,
    ) -> Color {
        let color = self.get_color();
        Color::rgb_linear(
            refit_to_range(
                membrane_potential,
                resting_potential,
                threshold_potential,
                0.0,
                color.r() * 2000.0,
            ),
            refit_to_range(
                membrane_potential,
                resting_potential,
                threshold_potential,
                0.0,
                color.g() * 2000.0,
            ),
            refit_to_range(
                membrane_potential,
                resting_potential,
                threshold_potential,
                0.0,
                color.b() * 2000.0,
            ),
        )
    }

    pub fn get_color_from_activation(&self, activation_percentage: f64) -> Color {
        let color = self.get_color();
        Color::rgb_linear(
            refit_to_range(
                activation_percentage as f32,
                0.0,
                1.0,
                0.0,
                color.r() * 2000.0,
            ),
            refit_to_range(
                activation_percentage as f32,
                0.0,
                1.0,
                0.0,
                color.g() * 2000.0,
            ),
            refit_to_range(
                activation_percentage as f32,
                0.0,
                1.0,
                0.0,
                color.b() * 2000.0,
            ),
        )
    }
}

fn refit_to_range(n: f32, start1: f32, stop1: f32, start2: f32, stop2: f32) -> f32 {
    ((n - start1) / (stop1 - start1)) * (stop2 - start2) + start2
}

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
                        emissive: Color::rgb_linear(23000.0, 9000.0, 3000.0),
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
                        emissive: Color::rgb_linear(23000.0, 9000.0, 3000.0),
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
                        emissive: Color::rgb_linear(23000.0, 9000.0, 3000.0),
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
                        emissive: Color::rgb_linear(3000.0, 23000.0, 9000.0),
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
                        emissive: Color::rgb_linear(23000.0, 9000.0, 3000.0),
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
                        emissive: Color::rgb_linear(23000.0, 9000.0, 3000.0),
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
