use bevy::prelude::*;

use super::{Neuron, NeuronVisualizer};

#[derive(Component, Debug, Reflect)]
pub struct LifNeuron {
    pub membrane_potential: f64,
    pub reset_potential: f64,
    pub threshold_potential: f64,
    pub resistance: f64,
    pub resting_potential: f64,
    pub refactory_period: f64,
    pub refactory_counter: f64,
}

impl Neuron for LifNeuron {
    fn update(&mut self, tau: f64) -> bool {
        if self.refactory_counter > 0.0 {
            self.refactory_counter -= tau;
            return false;
        }

        let delta_v = (self.resting_potential - self.membrane_potential) * tau;

        self.membrane_potential += delta_v;

        if self.membrane_potential > self.threshold_potential {
            self.membrane_potential = self.reset_potential;
            self.refactory_counter = self.refactory_period;
            return true;
        }

        false
    }

    fn get_membrane_potential(&self) -> f64 {
        self.membrane_potential
    }

    fn add_membrane_potential(&mut self, delta_v: f64) -> f64 {
        self.membrane_potential += delta_v;
        self.membrane_potential
    }
}

impl NeuronVisualizer for LifNeuron {
    fn activation_percent(&self) -> f64 {
        if self.membrane_potential < self.resting_potential {
            return 1.0;
        }

        refit_to_range(
            self.membrane_potential as f32,
            self.resting_potential as f32,
            self.threshold_potential as f32,
            0.0,
            1.0,
        ) as f64
    }
}

fn refit_to_range(n: f32, start1: f32, stop1: f32, start2: f32, stop2: f32) -> f32 {
    ((n - start1) / (stop1 - start1)) * (stop2 - start2) + start2
}
