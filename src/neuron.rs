use bevy::prelude::Component;

use crate::synapse::Synapse;

#[derive(Component, Debug)]
pub struct Neuron {
    pub membrane_potential: f64,  // V
    pub resting_potential: f64,   // V_rest
    pub reset_potential: f64,     // V_reset
    pub threshold_potential: f64, // V_thresh
    pub resistance: f64,          // R
    pub refractory_period: f32,   // t_ref
    pub refactory_counter: f32,   // t_ref_counter
}

impl Neuron {
    pub fn new() -> Self {
        Neuron {
            membrane_potential: -70.0,
            resting_potential: -70.0,
            reset_potential: -90.0,
            threshold_potential: -55.0,
            resistance: 1.3,
            refractory_period: 0.09,
            refactory_counter: 0.0,
        }
    }

    // V(t+1) = V(t) + Δt * (R - (V(t) - V_rest))
    pub fn tick(&mut self, time_step: f64) -> bool {
        if self.refactory_counter > 0.0 {
            self.refactory_counter -= time_step as f32;
            return false;
        }

        let delta_v =
            self.resistance * (self.resting_potential - self.membrane_potential) * time_step;
        self.membrane_potential += delta_v;

        if self.membrane_potential >= self.threshold_potential {
            self.membrane_potential = self.reset_potential;
            self.refactory_counter = self.refractory_period;
            return true;
        }

        false
    }

    pub fn set_membrane_potential(&mut self, membrane_potential: f64) {
        self.membrane_potential = membrane_potential;
    }

    pub fn apply_synapse(&mut self, synapse: &Synapse) {
        println!("Applying synapse: {:?}", synapse);
        println!("Before: {}", self.membrane_potential);
        self.membrane_potential += synapse.weight;
        println!("After: {}", self.membrane_potential);
    }
}
