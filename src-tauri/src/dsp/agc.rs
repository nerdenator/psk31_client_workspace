//! Automatic Gain Control

/// AGC with exponential attack/decay
pub struct Agc {
    target_level: f32,
    attack_rate: f32,
    decay_rate: f32,
    gain: f32,
    max_gain: f32,
    min_gain: f32,
}

impl Agc {
    pub fn new(target_level: f32) -> Self {
        Self {
            target_level,
            attack_rate: 0.01,
            decay_rate: 0.001,
            gain: 1.0,
            max_gain: 100.0,
            min_gain: 0.01,
        }
    }

    /// Process a sample through AGC
    pub fn process(&mut self, sample: f32) -> f32 {
        let output = sample * self.gain;
        let level = output.abs();

        // Adjust gain based on level vs target
        if level > self.target_level {
            self.gain *= 1.0 - self.attack_rate;
        } else {
            self.gain *= 1.0 + self.decay_rate;
        }

        self.gain = self.gain.clamp(self.min_gain, self.max_gain);
        output.clamp(-1.0, 1.0)
    }

    /// Get current gain value (useful for signal strength indication)
    pub fn current_gain(&self) -> f32 {
        self.gain
    }

    pub fn reset(&mut self) {
        self.gain = 1.0;
    }
}
