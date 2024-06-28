pub(crate) fn char_to_binary(c: char) -> [u8; 8] {
    let mut binary = [0; 8];
    for i in 0..8 {
        binary[i] = (c as u8 >> i) & 1;
    }
    binary
}

pub fn char_to_spike_train(c: char, time_frame: f64) -> Vec<f64> {
    let binary = char_to_binary(c);
    let mut spike_train = Vec::new();
    for i in 0..8 {
        if binary[i] == 1 {
            spike_train.push(i as f64 * time_frame);
        }
    }
    spike_train
}

pub fn string_to_spike_train(s: &str, time_frame: f64) -> Vec<f64> {
    let mut spike_train = Vec::new();
    for c in s.chars() {
        let mut char_spike_train = char_to_spike_train(c, time_frame / s.len() as f64);
        spike_train.append(&mut char_spike_train);
    }
    spike_train
}
