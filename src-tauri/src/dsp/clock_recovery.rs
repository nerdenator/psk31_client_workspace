//! Symbol timing recovery
//!
//! TODO: Implement Mueller-Muller timing recovery in Phase 5

/// Symbol clock recovery using Mueller-Muller timing error detector
pub struct ClockRecovery {
    samples_per_symbol: f64,
    mu: f64,           // Fractional sample offset (0.0 - 1.0)
    omega: f64,        // Current samples-per-symbol estimate
    gain_omega: f64,   // Timing gain
    gain_mu: f64,      // Phase gain
    last_sample: f32,
    last_symbol: f32,
    sample_count: f64,
}

impl ClockRecovery {
    /// Create a new clock recovery module
    ///
    /// For PSK-31 at 48kHz: samples_per_symbol = 48000 / 31.25 = 1536
    pub fn new(samples_per_symbol: f64) -> Self {
        Self {
            samples_per_symbol,
            mu: 0.5,
            omega: samples_per_symbol,
            gain_omega: 0.001,
            gain_mu: 0.01,
            last_sample: 0.0,
            last_symbol: 0.0,
            sample_count: 0.0,
        }
    }

    /// Process a sample, returns Some(symbol_value) at symbol decision points
    pub fn process(&mut self, sample: f32) -> Option<f32> {
        self.sample_count += 1.0;

        // Check if we've reached a symbol decision point
        if self.sample_count >= self.omega {
            self.sample_count -= self.omega;

            // Mueller-Muller timing error
            let timing_error = self.last_symbol * sample - self.last_sample * sample;

            // Update timing
            self.omega += self.gain_omega * timing_error as f64;
            self.mu += self.gain_mu * timing_error as f64;

            // Clamp omega to reasonable range
            self.omega = self.omega.clamp(
                self.samples_per_symbol * 0.9,
                self.samples_per_symbol * 1.1,
            );

            self.last_symbol = sample;
            self.last_sample = sample;

            Some(sample)
        } else {
            self.last_sample = sample;
            None
        }
    }

    /// Reset the clock recovery state
    pub fn reset(&mut self) {
        self.mu = 0.5;
        self.omega = self.samples_per_symbol;
        self.last_sample = 0.0;
        self.last_symbol = 0.0;
        self.sample_count = 0.0;
    }
}
