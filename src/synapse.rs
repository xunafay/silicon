pub struct Synapse {
    weight: f64,
    delay: u32,
}

impl Synapse {
    pub fn new() -> Self {
        Synapse {
            weight: 0.5,
            delay: 1,
        }
    }
}
