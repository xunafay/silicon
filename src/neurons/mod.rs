use bevy::prelude::Component;

#[derive(Component, Debug)]
pub struct Neuron {
    pub membrane_potential: f64,
    pub resting_potential: f64,
    pub reset_potential: f64,
    pub threshold_potential: f64,
    pub resistance: f64,
    pub refractory_period: f32,
    pub refactory_counter: f32,
}

#[derive(Component, Debug)]
pub struct OscillatingNeuron {
    pub frequency: f64,
    pub amplitude: f64,
}
