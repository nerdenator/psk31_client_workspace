//! PSK-31 Encoder — converts text to BPSK-31 audio samples
//!
//! Pipeline: text → Varicode bits → BPSK modulation → audio samples
//!
//! BPSK-31 encodes data by phase: a '1' bit keeps the same phase,
//! a '0' bit flips the phase by 180°. Think of it like Python's
//! `itertools.accumulate` over the bit stream, toggling a boolean.
//!
//! The raised cosine shaper smooths phase transitions so the signal
//! doesn't splatter across the band (like a key click filter for CW).

use crate::dsp::nco::Nco;
use crate::dsp::raised_cosine::RaisedCosineShaper;
use crate::modem::varicode::Varicode;

/// At 48 kHz sample rate and 31.25 baud, each symbol is exactly 1536 samples
const SAMPLES_PER_SYMBOL: usize = 1536;

/// Number of idle (phase-change) bits before data — lets the receiver lock on
const PREAMBLE_BITS: usize = 32;

/// Number of idle bits after data — clean ramp-down
const POSTAMBLE_BITS: usize = 32;

/// PSK-31 encoder: text in, audio samples out
pub struct Psk31Encoder {
    sample_rate: u32,
    carrier_freq: f64,
}

impl Psk31Encoder {
    pub fn new(sample_rate: u32, carrier_freq: f64) -> Self {
        Self {
            sample_rate,
            carrier_freq,
        }
    }

    /// Encode a text message into BPSK-31 audio samples.
    ///
    /// Returns a Vec<f32> of audio samples ready for playback at 48 kHz.
    pub fn encode(&self, text: &str) -> Vec<f32> {
        let bits = self.text_to_bits(text);
        self.bits_to_samples(&bits)
    }

    /// Convert text to a complete bit stream: preamble + varicode + postamble
    ///
    /// In BPSK-31, a '0' bit = phase change, '1' bit = no change.
    /// Varicode separators are '00' (two phase changes between characters).
    /// Preamble/postamble are all zeros (continuous phase changes).
    fn text_to_bits(&self, text: &str) -> Vec<bool> {
        let mut bits = Vec::new();

        // Preamble: continuous phase changes for receiver sync
        for _ in 0..PREAMBLE_BITS {
            bits.push(false); // 0 = phase change
        }

        // Encode each character
        for ch in text.chars() {
            if let Some(code_str) = Varicode::encode(ch) {
                let code_bits = Varicode::bits_from_str(code_str);
                bits.extend(code_bits);
                // Inter-character separator: two zeros
                bits.push(false);
                bits.push(false);
            }
            // Skip unsupported characters silently
        }

        // Postamble: clean ramp-down
        for _ in 0..POSTAMBLE_BITS {
            bits.push(false);
        }

        bits
    }

    /// Convert a bit stream to BPSK-modulated audio samples.
    ///
    /// For each bit:
    /// - Generate 1536 carrier samples from the NCO
    /// - Multiply by the raised cosine envelope
    /// - If the bit is '0', flip the carrier phase by π (180°)
    fn bits_to_samples(&self, bits: &[bool]) -> Vec<f32> {
        let mut nco = Nco::new(self.carrier_freq, self.sample_rate as f64);
        let shaper = RaisedCosineShaper::new(SAMPLES_PER_SYMBOL);

        let total_samples = bits.len() * SAMPLES_PER_SYMBOL;
        let mut samples = Vec::with_capacity(total_samples);

        for &bit in bits {
            // bit=false means phase change (BPSK convention)
            let phase_change = !bit;

            // Get the envelope shape for this symbol
            let envelope = shaper.generate_envelope(phase_change);

            // TODO: phase flip timing bug. The phase is flipped here at the START of the
            // symbol, but the envelope is 1.0 at the symbol start (and dips to 0 at the
            // midpoint). This causes a hard discontinuity at full amplitude — exactly what
            // raised cosine shaping is supposed to prevent. Correct PSK-31 requires the
            // phase to flip when the envelope crosses zero, i.e. the transition should span
            // the boundary between the previous and current symbol:
            //   - first half of current symbol: old phase × falling envelope (1→0)
            //   - second half of current symbol: new phase × rising envelope (0→1)
            // Fixing this requires restructuring bits_to_samples to look ahead at the next
            // bit. The current approach passes loopback tests but causes spectral splatter.
            if phase_change {
                nco.adjust_phase(std::f64::consts::PI);
            }

            // Generate carrier samples shaped by the envelope
            for i in 0..SAMPLES_PER_SYMBOL {
                let carrier = nco.next();
                samples.push(carrier * envelope[i]);
            }
        }

        samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modem::varicode::VaricodeDecoder;

    #[test]
    fn test_encode_empty_text() {
        let encoder = Psk31Encoder::new(48000, 1500.0);
        let samples = encoder.encode("");

        // Should have preamble + postamble only
        let expected_bits = PREAMBLE_BITS + POSTAMBLE_BITS;
        let expected_samples = expected_bits * SAMPLES_PER_SYMBOL;
        assert_eq!(samples.len(), expected_samples);
    }

    #[test]
    fn test_encode_single_char() {
        let encoder = Psk31Encoder::new(48000, 1500.0);
        let samples = encoder.encode("e");

        // 'e' = "11" (2 bits) + "00" separator (2 bits)
        // Total: 32 preamble + 2 char + 2 separator + 32 postamble = 68 bits
        let expected_samples = 68 * SAMPLES_PER_SYMBOL;
        assert_eq!(samples.len(), expected_samples);
    }

    #[test]
    fn test_encode_known_text_bit_count() {
        let encoder = Psk31Encoder::new(48000, 1500.0);

        // "CQ" = C="10101101" (8 bits) + 00 + Q="111011101" (9 bits) + 00
        // Total data bits: 8 + 2 + 9 + 2 = 21
        // Total: 32 + 21 + 32 = 85 bits
        let samples = encoder.encode("CQ");
        let expected_samples = 85 * SAMPLES_PER_SYMBOL;
        assert_eq!(samples.len(), expected_samples);
    }

    #[test]
    fn test_samples_in_valid_range() {
        let encoder = Psk31Encoder::new(48000, 1500.0);
        let samples = encoder.encode("TEST");

        for (i, &s) in samples.iter().enumerate() {
            assert!(
                s >= -1.0 && s <= 1.0,
                "Sample {} out of range: {}",
                i,
                s
            );
        }
    }

    #[test]
    fn test_preamble_is_not_silent() {
        let encoder = Psk31Encoder::new(48000, 1500.0);
        let samples = encoder.encode("A");

        // The preamble should have audible carrier, not silence
        let preamble_samples = &samples[..PREAMBLE_BITS * SAMPLES_PER_SYMBOL];
        let max_amplitude: f32 = preamble_samples
            .iter()
            .map(|s| s.abs())
            .fold(0.0f32, f32::max);

        assert!(
            max_amplitude > 0.1,
            "Preamble seems silent: max amplitude = {}",
            max_amplitude
        );
    }

    #[test]
    fn test_encode_decode_loopback() {
        // Encode "HI" → detect phase transitions → decode with VaricodeDecoder
        let encoder = Psk31Encoder::new(48000, 1500.0);
        let samples = encoder.encode("HI");

        // Extract bits by detecting phase changes between symbols.
        // We compare the sign of the carrier at a consistent point in each
        // symbol (quarter-way through, to avoid envelope nulls).
        let check_offset = SAMPLES_PER_SYMBOL / 4;
        let num_symbols = samples.len() / SAMPLES_PER_SYMBOL;

        let mut bits = Vec::new();
        let mut prev_sign = samples[check_offset] >= 0.0;

        for sym_idx in 1..num_symbols {
            let sample_idx = sym_idx * SAMPLES_PER_SYMBOL + check_offset;
            let current_sign = samples[sample_idx] >= 0.0;
            // Phase change = bit 0, no change = bit 1
            bits.push(current_sign == prev_sign);
            prev_sign = current_sign;
        }

        // Skip preamble bits (they're all zeros / phase changes)
        // and decode the data portion
        let data_bits = &bits[PREAMBLE_BITS - 1..]; // -1 because we lost the first bit in diff detection

        let mut decoder = VaricodeDecoder::new();
        let mut decoded = String::new();

        for &bit in data_bits {
            if let Some(ch) = decoder.push_bit(bit) {
                decoded.push(ch);
            }
        }

        assert!(
            decoded.contains("HI"),
            "Expected decoded text to contain 'HI', got: '{}'",
            decoded
        );
    }
}
