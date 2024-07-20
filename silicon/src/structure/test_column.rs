use bevy::{
    asset::Assets,
    color::{Color, LinearRgba},
    pbr::{PbrBundle, StandardMaterial},
    prelude::{Commands, ResMut},
    render::{
        mesh::{Mesh, Meshable},
        view::Visibility,
    },
    transform::components::Transform,
};
use bevy_math::primitives::Cuboid;
use bevy_rapier3d::geometry::Collider;
use neurons::izhikevich::IzhikevichNeuron;
use simulator::SimpleSpikeRecorder;
use synapses::AllowSynapses;

use super::layer::ColumnLayer;

pub struct TestColumn {}

impl TestColumn {
    pub fn create(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        let mesh = meshes.add(Cuboid::new(0.5, 0.5, 0.5).mesh());

        for x in 0..2 {
            for y in 0..2 {
                for z in 0..1 {
                    let leaky_neuron_material = materials.add(StandardMaterial {
                        emissive: LinearRgba::rgb(23.0, 9.0, 3.0),
                        ..Default::default()
                    });

                    commands.spawn((
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
                        Collider::cuboid(0.25, 0.25, 0.25),
                        ColumnLayer::L1,
                        AllowSynapses,
                        SimpleSpikeRecorder::default(),
                    ));
                }
            }
        }

        for x in 0..2 {
            for y in 0..2 {
                for z in 0..1 {
                    let leaky_neuron_material = materials.add(StandardMaterial {
                        emissive: LinearRgba::rgb(23.0, 9.0, 3.0),
                        ..Default::default()
                    });

                    commands.spawn((
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
                        Collider::cuboid(0.25, 0.25, 0.25),
                        ColumnLayer::L4,
                        AllowSynapses,
                        SimpleSpikeRecorder::default(),
                    ));
                }
            }
        }
    }
}
