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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loud_signal_reduces_gain() {
        let mut agc = Agc::new(0.5);
        let initial_gain = agc.current_gain();

        // Feed loud signal (amplitude 1.0, above target 0.5)
        for _ in 0..1000 {
            agc.process(1.0);
        }

        assert!(
            agc.current_gain() < initial_gain,
            "Gain should decrease for loud signals"
        );
    }

    #[test]
    fn test_quiet_signal_increases_gain() {
        let mut agc = Agc::new(0.5);
        let initial_gain = agc.current_gain();

        // Feed very quiet signal
        for _ in 0..1000 {
            agc.process(0.01);
        }

        assert!(
            agc.current_gain() > initial_gain,
            "Gain should increase for quiet signals"
        );
    }

    #[test]
    fn test_gain_stays_within_bounds() {
        let mut agc = Agc::new(0.5);

        // Drive gain up with silence
        for _ in 0..100_000 {
            agc.process(0.0001);
        }
        assert!(agc.current_gain() <= 100.0);

        // Drive gain down with max amplitude
        agc.reset();
        for _ in 0..100_000 {
            agc.process(1.0);
        }
        assert!(agc.current_gain() >= 0.01);
    }

    #[test]
    fn test_output_clamped() {
        let mut agc = Agc::new(0.5);
        // Pump gain up first with quiet signal
        for _ in 0..10_000 {
            agc.process(0.001);
        }
        // Now feed a loud sample — output should be clamped
        let output = agc.process(1.0);
        assert!(output >= -1.0 && output <= 1.0);
    }

    #[test]
    fn test_reset() {
        let mut agc = Agc::new(0.5);
        for _ in 0..5000 {
            agc.process(0.01);
        }
        assert_ne!(agc.current_gain(), 1.0);
        agc.reset();
        assert_eq!(agc.current_gain(), 1.0);
    }
}
