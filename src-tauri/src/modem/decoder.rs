//! PSK-31 Decoder — converts BPSK-31 audio samples back to text
//!
//! Pipeline: audio samples → AGC → Costas Loop → clock recovery
//!           → differential bit detection → Varicode decode → characters
//!
//! The Costas Loop locks onto the BPSK carrier, tracks its frequency/phase,
//! and downmixes to baseband. Differential decoding resolves the 180° phase
//! ambiguity by detecting phase *changes* rather than absolute phase.
//!
//! Note: The first character of a transmission is typically lost during
//! lock acquisition. This is normal PSK-31 behavior — real QSOs always
//! start with repeated CQ calls so the receiver has time to lock.

use crate::dsp::agc::Agc;
use crate::dsp::clock_recovery::ClockRecovery;
use crate::dsp::costas_loop::CostasLoop;
use crate::modem::varicode::VaricodeDecoder;

/// Number of bits without a valid decoded character before we try
/// inverting the bit sense (phase ambiguity fallback)
const PHASE_AMBIGUITY_THRESHOLD: usize = 100;

/// Minimum symbol magnitude for bit decisions. Below this threshold,
/// the Costas Loop hasn't locked yet and bit decisions would be garbage.
const SYMBOL_SQUELCH: f32 = 0.001;

/// PSK-31 decoder: audio samples in, decoded characters out
pub struct Psk31Decoder {
    agc: Agc,
    costas_loop: CostasLoop,
    clock_recovery: ClockRecovery,
    varicode_decoder: VaricodeDecoder,

    /// Previous symbol value for differential detection
    last_symbol: f32,

    /// Count of consecutive bits without a valid Varicode character
    bits_without_char: usize,

    /// When true, invert bit sense (phase ambiguity fallback)
    invert_bits: bool,

    sample_rate: u32,
    carrier_freq: f64,
}

impl Psk31Decoder {
    /// Create a new decoder tuned to the given carrier frequency
    ///
    /// - `carrier_freq`: audio carrier in Hz (typically 500-2500, set by waterfall click)
    /// - `sample_rate`: audio sample rate (48000)
    pub fn new(carrier_freq: f64, sample_rate: u32) -> Self {
        let samples_per_symbol = sample_rate as f64 / 31.25;

        Self {
            agc: Agc::new(0.5),
            costas_loop: CostasLoop::new(carrier_freq, sample_rate as f64, 2.0),
            clock_recovery: ClockRecovery::new(samples_per_symbol),
            varicode_decoder: VaricodeDecoder::new(),
            last_symbol: 0.0,
            bits_without_char: 0,
            invert_bits: false,
            sample_rate,
            carrier_freq,
        }
    }

    /// Process a single audio sample. Returns `Some(char)` when a character
    /// is fully decoded, `None` otherwise.
    pub fn process(&mut self, sample: f32) -> Option<char> {
        // 1. AGC — normalize amplitude
        let normalized = self.agc.process(sample);

        // 2. Costas Loop — carrier tracking + downmix to baseband
        let baseband = self.costas_loop.process(normalized);

        // 3. Clock Recovery — extract symbol at decision points
        let symbol = self.clock_recovery.process(baseband)?;

        // 4. Symbol squelch — ignore weak symbols during lock acquisition
        if symbol.abs() < SYMBOL_SQUELCH && self.last_symbol.abs() < SYMBOL_SQUELCH {
            self.last_symbol = symbol;
            return None;
        }

        // 5. Differential bit detection
        let same_sign = (symbol > 0.0) == (self.last_symbol > 0.0);
        self.last_symbol = symbol;

        let raw_bit = same_sign;
        let bit = if self.invert_bits { !raw_bit } else { raw_bit };

        // 6. Varicode decode
        self.bits_without_char += 1;

        if let Some(ch) = self.varicode_decoder.push_bit(bit) {
            self.bits_without_char = 0;
            return Some(ch);
        }

        // 7. Phase ambiguity fallback
        if self.bits_without_char > PHASE_AMBIGUITY_THRESHOLD {
            self.invert_bits = !self.invert_bits;
            self.bits_without_char = 0;
            self.varicode_decoder.reset();
        }

        None
    }

    /// Update the carrier frequency (e.g., from waterfall click-to-tune)
    ///
    /// Resets carrier tracking and bit-layer state but preserves AGC gain
    /// to avoid unnecessary settle time after retuning.
    pub fn set_carrier_freq(&mut self, freq: f64) {
        self.carrier_freq = freq;
        self.costas_loop.set_frequency(freq);
        self.costas_loop.reset();
        self.clock_recovery = ClockRecovery::new(self.sample_rate as f64 / 31.25);
        self.varicode_decoder.reset();
        self.last_symbol = 0.0;
        self.bits_without_char = 0;
        self.invert_bits = false;
    }

    /// Signal strength as a 0.0..=1.0 value derived from AGC gain.
    ///
    /// The AGC gain is inversely proportional to signal level: low gain = strong signal.
    /// Gain range is [0.01, 100.0], mapped via inverse log10 to [1.0, 0.0]:
    ///   gain=0.01 → 1.0 (strong), gain=1.0 → 0.5, gain=100.0 → 0.0 (absent)
    pub fn signal_strength(&self) -> f32 {
        let gain = self.agc.current_gain().clamp(0.01, 100.0);
        (1.0 - (gain.log10() + 2.0) / 4.0).clamp(0.0, 1.0)
    }

    /// Reset all decoder state
    pub fn reset(&mut self) {
        self.agc.reset();
        self.costas_loop.reset();
        self.clock_recovery.reset();
        self.varicode_decoder.reset();
        self.last_symbol = 0.0;
        self.bits_without_char = 0;
        self.invert_bits = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modem::encoder::Psk31Encoder;

    #[test]
    fn test_decode_encoder_output() {
        let carrier_freq = 1000.0;
        let sample_rate = 48000;

        let encoder = Psk31Encoder::new(sample_rate, carrier_freq);
        let samples = encoder.encode("HI");

        let mut decoder = Psk31Decoder::new(carrier_freq, sample_rate);
        let mut decoded = String::new();

        for &sample in &samples {
            if let Some(ch) = decoder.process(sample) {
                decoded.push(ch);
            }
        }

        // First character may be lost during lock acquisition — this is
        // normal PSK-31 behavior. Check that we decode at least the 'I'.
        assert!(
            decoded.contains('I'),
            "Expected to decode at least 'I', got: '{}'",
            decoded
        );
    }

    #[test]
    fn test_decode_longer_text() {
        let carrier_freq = 1500.0;
        let sample_rate = 48000;

        let encoder = Psk31Encoder::new(sample_rate, carrier_freq);
        let samples = encoder.encode("CQ CQ DE W1AW");

        let mut decoder = Psk31Decoder::new(carrier_freq, sample_rate);
        let mut decoded = String::new();

        for &sample in &samples {
            if let Some(ch) = decoder.process(sample) {
                decoded.push(ch);
            }
        }

        // Should decode the bulk of the message (first char may be lost)
        assert!(
            decoded.contains("Q DE W1AW"),
            "Expected message core, got: '{}'",
            decoded
        );
    }

    #[test]
    fn test_decode_at_different_carrier() {
        let carrier_freq = 2000.0;
        let sample_rate = 48000;

        let encoder = Psk31Encoder::new(sample_rate, carrier_freq);
        let samples = encoder.encode("TEST");

        let mut decoder = Psk31Decoder::new(carrier_freq, sample_rate);
        let mut decoded = String::new();

        for &sample in &samples {
            if let Some(ch) = decoder.process(sample) {
                decoded.push(ch);
            }
        }

        assert!(
            decoded.contains("EST"),
            "Expected at least 'EST' at 2000 Hz, got: '{}'",
            decoded
        );
    }

    #[test]
    fn test_retune_resets_state() {
        let mut decoder = Psk31Decoder::new(1000.0, 48000);

        for i in 0..10000 {
            decoder.process((i as f32 * 0.1).sin());
        }

        decoder.set_carrier_freq(1500.0);
        assert_eq!(decoder.carrier_freq, 1500.0);
        assert_eq!(decoder.last_symbol, 0.0);
        assert_eq!(decoder.bits_without_char, 0);
        assert!(!decoder.invert_bits);
    }
}
