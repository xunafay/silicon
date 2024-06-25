use bevy::prelude::*;
use uom::si::{f64::Time as SiTime, time::second};

use super::Neuron;

#[derive(Component, Debug)]
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
    fn update(&mut self, tau: SiTime) -> bool {
        if self.refactory_counter > 0.0 {
            self.refactory_counter -= tau.get::<second>();
            return false;
        }

        let delta_v = (self.resting_potential - self.membrane_potential) * tau.get::<second>()
            / self.resistance;

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
