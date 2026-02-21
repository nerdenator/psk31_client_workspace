//! Integration tests: TX encoder → RX decoder loopback
//!
//! These tests verify the full PSK-31 pipeline by encoding text with the
//! encoder, then feeding those audio samples through the decoder and
//! checking that the original text is recovered.

use psk31_client_lib::modem::decoder::Psk31Decoder;
use psk31_client_lib::modem::encoder::Psk31Encoder;

/// Helper: encode text, decode it, return the decoded string
fn loopback(text: &str, carrier_freq: f64, sample_rate: u32) -> String {
    let encoder = Psk31Encoder::new(sample_rate, carrier_freq);
    let samples = encoder.encode(text);

    let mut decoder = Psk31Decoder::new(carrier_freq, sample_rate);
    let mut decoded = String::new();

    for &sample in &samples {
        if let Some(ch) = decoder.process(sample) {
            decoded.push(ch);
        }
    }

    decoded
}

#[test]
fn test_loopback_short_message() {
    let decoded = loopback("HI", 1000.0, 48000);
    // First char may be lost during Costas Loop lock acquisition
    assert!(
        decoded.contains('I'),
        "Expected at least 'I', got: '{decoded}'"
    );
}

#[test]
fn test_loopback_cq_call() {
    let decoded = loopback("CQ CQ DE W1AW", 1000.0, 48000);
    assert!(
        decoded.contains("Q DE W1AW"),
        "Expected message core, got: '{decoded}'"
    );
}

#[test]
fn test_loopback_at_different_carriers() {
    // PSK-31 typical range: 500-2500 Hz audio passband
    for &freq in &[1000.0, 1500.0, 2000.0, 2500.0] {
        let decoded = loopback("TEST MSG", freq, 48000);
        assert!(
            decoded.contains("EST MSG"),
            "Failed at {freq} Hz, got: '{decoded}'"
        );
    }
}

#[test]
fn test_loopback_lowercase_and_punctuation() {
    let decoded = loopback("hello, world!", 1000.0, 48000);
    assert!(
        decoded.contains("ello, world!"),
        "Expected lowercase + punctuation, got: '{decoded}'"
    );
}

#[test]
fn test_loopback_with_small_frequency_offset() {
    // Encode at 1000 Hz, decode at 1001 Hz (1 Hz offset)
    // The Costas Loop should pull in and track the signal
    let encoder = Psk31Encoder::new(48000, 1000.0);
    let samples = encoder.encode("CQ CQ DE W1AW");

    let mut decoder = Psk31Decoder::new(1001.0, 48000);
    let mut decoded = String::new();

    for &sample in &samples {
        if let Some(ch) = decoder.process(sample) {
            decoded.push(ch);
        }
    }

    assert!(
        decoded.contains("DE W1AW"),
        "Expected to decode with 1 Hz offset, got: '{decoded}'"
    );
}
