use bevy::{prelude::Component, reflect::Reflect};

use super::{Neuron, NeuronVisualizer};

#[derive(Component, Debug, Reflect)]
pub struct IzhikevichNeuron {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub v: f64,
    pub u: f64,
    pub synapse_weight_multiplier: f64,
}

impl Neuron for IzhikevichNeuron {
    fn update(&mut self, tau: f64) -> bool {
        let v = self.v + tau * (0.04 * self.v * self.v + 5.0 * self.v + 140.0 - self.u) + 0.0;
        let u = self.u + tau * self.a * (self.b * self.v - self.u);
        self.v = v;
        self.u = u;
        if self.v >= 30.0 {
            self.v = self.c;
            self.u += self.d;
            return true;
        }

        false
    }

    fn get_membrane_potential(&self) -> f64 {
        self.v
    }

    fn add_membrane_potential(&mut self, delta_v: f64) -> f64 {
        self.v += delta_v * self.synapse_weight_multiplier;
        self.v
    }
}

impl NeuronVisualizer for IzhikevichNeuron {
    fn activation_percent(&self) -> f64 {
        if self.v < -65.0 {
            return 1.0;
        }

        (self.v + 65.0) / 30.0
    }
}
