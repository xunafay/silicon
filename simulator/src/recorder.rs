use bevy::prelude::{Entity, Query, Res};
use bevy_trait_query::One;
use silicon_core::{Clock, Neuron, ValueRecorder, ValueRecorderConfig};
use synapses::Synapse;

pub(crate) fn record_membrane_potential(
    mut neurons_query: Query<(Entity, One<&dyn Neuron>, &mut ValueRecorder)>,
    clock: Res<Clock>,
) {
    for (_entity, neuron, mut value_recorder) in neurons_query.iter_mut() {
        value_recorder
            .values
            .push((clock.time, neuron.get_membrane_potential()));
    }
}

pub(crate) fn record_synapse_weight(
    mut synapses_query: Query<(Entity, One<&dyn Synapse>, &mut ValueRecorder)>,
    clock: Res<Clock>,
) {
    for (_, synapse, mut value_recorder) in synapses_query.iter_mut() {
        value_recorder
            .values
            .push((clock.time, synapse.get_weight()));
    }
}

pub(crate) fn clean_recorder_history(
    mut recorders: Query<&mut ValueRecorder>,
    clock: Res<Clock>,
    history_config: Res<ValueRecorderConfig>,
) {
    for mut recorder in recorders.iter_mut() {
        recorder.values = recorder
            .values
            .iter()
            .filter(|(time, _)| clock.time - time < history_config.window_size as f64)
            .cloned()
            .collect();
    }
}
