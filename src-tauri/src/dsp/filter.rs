//! FIR filter implementation

/// FIR filter for bandpass/lowpass filtering
pub struct FirFilter {
    coefficients: Vec<f32>,
    delay_line: Vec<f32>,
    position: usize,
}

impl FirFilter {
    /// Create a new FIR filter with the given coefficients
    pub fn new(coefficients: Vec<f32>) -> Self {
        let len = coefficients.len();
        Self {
            coefficients,
            delay_line: vec![0.0; len],
            position: 0,
        }
    }

    /// Create a simple lowpass filter using windowed sinc
    pub fn lowpass(cutoff_freq: f32, sample_rate: f32, num_taps: usize) -> Self {
        let normalized_cutoff = cutoff_freq / sample_rate;
        let mut coefficients = vec![0.0; num_taps];
        let middle = num_taps / 2;

        for i in 0..num_taps {
            let n = i as f32 - middle as f32;
            if n == 0.0 {
                coefficients[i] = 2.0 * normalized_cutoff;
            } else {
                coefficients[i] = (2.0 * std::f32::consts::PI * normalized_cutoff * n).sin()
                    / (std::f32::consts::PI * n);
            }

            // Apply Hanning window
            let window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / num_taps as f32).cos());
            coefficients[i] *= window;
        }

        // Normalize
        let sum: f32 = coefficients.iter().sum();
        for c in &mut coefficients {
            *c /= sum;
        }

        Self::new(coefficients)
    }

    /// Process a single sample through the filter
    pub fn process(&mut self, sample: f32) -> f32 {
        self.delay_line[self.position] = sample;

        let mut output = 0.0;
        let len = self.coefficients.len();

        for i in 0..len {
            let delay_idx = (self.position + len - i) % len;
            output += self.coefficients[i] * self.delay_line[delay_idx];
        }

        self.position = (self.position + 1) % len;
        output
    }

    /// Reset the filter state
    pub fn reset(&mut self) {
        self.delay_line.fill(0.0);
        self.position = 0;
    }
}
