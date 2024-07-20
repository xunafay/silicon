use bevy::{prelude::Entity, reflect::Reflect};

#[derive(Debug, Clone, Reflect)]
pub struct PopulationEncoder {
    pub neurons: Vec<Entity>,
}

impl PopulationEncoder {
    /// sample_rate is a value between 0.0 and 1.0 that determines the percentage of neurons to include in the population
    pub fn from_sample_rate(neurons: &Vec<Entity>, sample_rate: f64) -> Self {
        let selected_neurons = neurons
            .iter()
            .enumerate()
            .filter(|(_, _)| rand::random::<f64>() < sample_rate)
            .map(|(_, neuron)| *neuron)
            .collect();

        PopulationEncoder {
            neurons: selected_neurons,
        }
    }
}
