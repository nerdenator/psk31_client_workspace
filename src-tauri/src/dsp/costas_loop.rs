//! Costas Loop for BPSK carrier tracking
//!
//! TODO: Implement in Phase 5 (RX path)

use super::{nco::Nco, filter::FirFilter};

/// Costas loop for BPSK carrier tracking and demodulation
pub struct CostasLoop {
    nco: Nco,
    lpf_i: FirFilter,
    lpf_q: FirFilter,
    loop_bandwidth: f64,
    proportional_gain: f64,
    integral_gain: f64,
    integrator: f64,
}

impl CostasLoop {
    /// Create a new Costas loop
    pub fn new(carrier_freq: f64, sample_rate: f64, loop_bandwidth: f64) -> Self {
        let nco = Nco::new(carrier_freq, sample_rate);

        // Low-pass filter for I/Q arms (cutoff well below symbol rate)
        let lpf_i = FirFilter::lowpass(50.0, sample_rate as f32, 63);
        let lpf_q = FirFilter::lowpass(50.0, sample_rate as f32, 63);

        // PLL gains (second-order loop with damping = 0.707)
        let damping = 0.707;
        let omega_n = loop_bandwidth * 8.0 * damping / (4.0 * damping * damping + 1.0);
        let proportional_gain = 2.0 * damping * omega_n;
        let integral_gain = omega_n * omega_n;

        Self {
            nco,
            lpf_i,
            lpf_q,
            loop_bandwidth,
            proportional_gain,
            integral_gain,
            integrator: 0.0,
        }
    }

    /// Process a single sample, returns the demodulated baseband I value
    pub fn process(&mut self, sample: f32) -> f32 {
        // Mix with local oscillator
        let (nco_i, nco_q) = self.nco.next_iq();
        let mixed_i = sample * nco_i;
        let mixed_q = sample * nco_q;

        // Low-pass filter both arms
        let filtered_i = self.lpf_i.process(mixed_i);
        let filtered_q = self.lpf_q.process(mixed_q);

        // Phase error detector for BPSK: e = I * Q
        let phase_error = (filtered_i * filtered_q) as f64;

        // Loop filter (PI controller)
        self.integrator += self.integral_gain * phase_error;
        let correction = self.proportional_gain * phase_error + self.integrator;

        // Adjust NCO
        self.nco.adjust_phase(correction);

        filtered_i
    }

    /// Set the carrier frequency
    pub fn set_frequency(&mut self, freq: f64) {
        self.nco.set_frequency(freq);
    }

    /// Reset the loop state
    pub fn reset(&mut self) {
        self.nco.reset();
        self.lpf_i.reset();
        self.lpf_q.reset();
        self.integrator = 0.0;
    }
}
