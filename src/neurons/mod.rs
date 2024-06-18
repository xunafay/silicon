use bevy::prelude::Component;
use uom::{
    si::{
        electric_potential::millivolt,
        electrical_resistance::ohm,
        f64::{Capacitance, ElectricPotential, ElectricalResistance, Frequency, Time},
        time,
    },
    ConstZero,
};

#[derive(Component, Debug)]
pub struct LeakyIntegrateNeuron1D {
    pub membrane_potential: ElectricPotential,
    pub reset_potential: ElectricPotential,
    pub resting_potential: ElectricPotential,
    pub threshold_potential: ElectricPotential,
    pub resistance: ElectricalResistance,
    pub refractory_period: Time,
    pub refactory_counter: Time,
}

impl LeakyIntegrateNeuron1D {
    pub fn new(
        membrane_potential: ElectricPotential,
        reset_potential: ElectricPotential,
        resting_potential: ElectricPotential,
        threshold_potential: ElectricPotential,
        resistance: ElectricalResistance,
        refractory_period: Time,
    ) -> Self {
        Self {
            membrane_potential,
            reset_potential,
            resting_potential,
            threshold_potential,
            resistance,
            refractory_period,
            refactory_counter: Time::ZERO,
        }
    }

    pub fn tick(&mut self, time_step: Time) -> bool {
        if self.refactory_counter > Time::ZERO {
            self.refactory_counter -= time_step;
            return false;
        }

        let delta_v = (self.resistance.get::<ohm>()
            * (self.resting_potential.get::<millivolt>()
                - self.membrane_potential.get::<millivolt>()))
            / time_step.get::<time::second>();
        self.membrane_potential += ElectricPotential::new::<millivolt>(delta_v);

        if self.membrane_potential >= self.threshold_potential {
            self.membrane_potential = self.reset_potential;
            self.refactory_counter = self.refractory_period;
            return true;
        }

        false
    }
}

pub struct AdaptiveLeakyIntegrateNeuron1D {}

#[derive(Component, Debug)]
pub struct OscillatingNeuron1D {
    pub membrane_potential: ElectricPotential,
    pub reset_potential: ElectricPotential,
    pub resting_potential: ElectricPotential,
    pub threshold_potential: ElectricPotential,
    pub resistance: ElectricalResistance,
    pub refractory_period: Time,
    pub refactory_counter: Time,
    pub frequency: f64,
    pub amplitude: f64,
}

impl OscillatingNeuron1D {
    pub fn new(
        membrane_potential: ElectricPotential,
        reset_potential: ElectricPotential,
        resting_potential: ElectricPotential,
        threshold_potential: ElectricPotential,
        resistance: ElectricalResistance,
        refractory_period: Time,
        frequency: f64,
        amplitude: f64,
    ) -> Self {
        Self {
            membrane_potential,
            reset_potential,
            resting_potential,
            threshold_potential,
            resistance,
            refractory_period,
            refactory_counter: Time::ZERO,
            frequency,
            amplitude,
        }
    }

    pub fn tick(&mut self, time_step: Time) -> bool {
        if self.refactory_counter > Time::ZERO {
            self.refactory_counter -= time_step;
            return false;
        }

        let delta_v = (self.resistance.get::<ohm>()
            * (self.resting_potential.get::<millivolt>()
                - self.membrane_potential.get::<millivolt>()))
            / time_step.get::<time::second>()
            + self.amplitude * ((2.0 * std::f32::consts::PI) as f64 * self.frequency).sin() as f64;
        self.membrane_potential += ElectricPotential::new::<millivolt>(delta_v);

        if self.membrane_potential >= self.threshold_potential {
            self.membrane_potential = self.reset_potential;
            self.refactory_counter = self.refractory_period;
            return true;
        }

        false
    }
}
