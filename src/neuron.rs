pub struct Neuron {
    membrane_potential: f64,
    resting_potential: f64,
    reset_potential: f64,
    threshold_potential: f64,
    resistance: f64,
}

impl Neuron {
    pub fn new() -> Self {
        Neuron {
            membrane_potential: -70.0,
            resting_potential: -70.0,
            reset_potential: -80.0,
            threshold_potential: -55.0,
            resistance: 10.0,
        }
    }

    pub fn update(&mut self, input_current: f64, time_step: f64) -> bool {
        let delta_v = (input_current * self.resistance + self.resting_potential
            - self.membrane_potential)
            * time_step;
        self.membrane_potential += delta_v;

        if self.membrane_potential >= self.threshold_potential {
            self.membrane_potential = self.reset_potential;
            return true;
        }

        false
    }
}
