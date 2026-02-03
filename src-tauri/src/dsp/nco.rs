//! Numerically Controlled Oscillator

use std::f64::consts::PI;

/// Numerically Controlled Oscillator for generating carrier signals
pub struct Nco {
    phase: f64,
    phase_increment: f64,
    sample_rate: f64,
}

impl Nco {
    /// Create a new NCO with the given frequency and sample rate
    pub fn new(frequency: f64, sample_rate: f64) -> Self {
        let phase_increment = 2.0 * PI * frequency / sample_rate;
        Self {
            phase: 0.0,
            phase_increment,
            sample_rate,
        }
    }

    /// Set the oscillator frequency
    pub fn set_frequency(&mut self, frequency: f64) {
        self.phase_increment = 2.0 * PI * frequency / self.sample_rate;
    }

    /// Get the current frequency
    pub fn frequency(&self) -> f64 {
        self.phase_increment * self.sample_rate / (2.0 * PI)
    }

    /// Adjust phase by a delta (used by PLLs for frequency correction)
    pub fn adjust_phase(&mut self, delta: f64) {
        self.phase += delta;
        self.wrap_phase();
    }

    /// Generate the next I/Q sample pair (cos, sin)
    pub fn next_iq(&mut self) -> (f32, f32) {
        let i = self.phase.cos() as f32;
        let q = self.phase.sin() as f32;
        self.phase += self.phase_increment;
        self.wrap_phase();
        (i, q)
    }

    /// Generate the next real sample (cosine only)
    pub fn next(&mut self) -> f32 {
        let sample = self.phase.cos() as f32;
        self.phase += self.phase_increment;
        self.wrap_phase();
        sample
    }

    /// Reset phase to zero
    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    fn wrap_phase(&mut self) {
        while self.phase >= 2.0 * PI {
            self.phase -= 2.0 * PI;
        }
        while self.phase < 0.0 {
            self.phase += 2.0 * PI;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nco_frequency() {
        let mut nco = Nco::new(1000.0, 48000.0);

        // Generate one full cycle worth of samples
        let samples_per_cycle = 48000.0 / 1000.0; // 48 samples
        let mut samples = Vec::new();

        for _ in 0..(samples_per_cycle as usize * 2) {
            samples.push(nco.next());
        }

        // Check that we have roughly 2 complete cycles
        let zero_crossings: usize = samples
            .windows(2)
            .filter(|w| (w[0] >= 0.0 && w[1] < 0.0) || (w[0] < 0.0 && w[1] >= 0.0))
            .count();

        // 2 cycles = 4 zero crossings
        assert_eq!(zero_crossings, 4);
    }
}
