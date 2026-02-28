//! Pure decoding: CAT wire string + command context → CatResponse.
//!
//! No I/O, no side effects. The `cmd` parameter tells us which fields
//! to expect in the response — the FT-991A uses the same prefix for
//! queries and replies so we need the context to know what we're parsing.

use crate::domain::{Psk31Error, Psk31Result};

use super::{CatCommand, CatResponse, MODE_TABLE};

/// Decode a raw response string from the radio into a typed CatResponse.
///
/// `response` is the string received after stripping any command echo.
/// `cmd` is the command that was sent, used to pick the right parser.
///
/// Returns `Err` if the response is `"?"` (radio NAK) or cannot be parsed.
pub fn decode(response: &str, cmd: &CatCommand) -> Psk31Result<CatResponse> {
    use CatCommand::*;

    // The radio returns "?" when it doesn't understand or rejects a command.
    if response.trim_end_matches(';') == "?" || response == "?" {
        return Err(Psk31Error::Cat(format!(
            "Radio NAK for command {cmd:?}: response was '?'"
        )));
    }

    match cmd {
        GetFrequencyA => parse_frequency(response),
        SetFrequencyA(_) => expect_ack(response, cmd),
        GetMode => parse_mode(response),
        SetMode(_) => expect_ack(response, cmd),
        PttOn | PttOff => expect_ack(response, cmd),
        GetTxPower => parse_tx_power(response),
        SetTxPower(_) => expect_ack(response, cmd),
    }
}

/// Parse `"FA00014070000;"` → `FrequencyHz(14_070_000)`
fn parse_frequency(response: &str) -> Psk31Result<CatResponse> {
    let trimmed = response.trim().trim_end_matches(';');
    if !trimmed.starts_with("FA") || trimmed.len() < 13 {
        return Err(Psk31Error::Cat(format!(
            "Invalid frequency response: '{response}'"
        )));
    }
    let digits = &trimmed[2..13];
    let hz = digits
        .parse::<u64>()
        .map_err(|e| Psk31Error::Cat(format!("Failed to parse frequency '{digits}': {e}")))?;
    Ok(CatResponse::FrequencyHz(hz))
}

/// Parse `"MD0C;"` → `Mode("DATA-USB")`
fn parse_mode(response: &str) -> Psk31Result<CatResponse> {
    let trimmed = response.trim().trim_end_matches(';');
    if !trimmed.starts_with("MD0") || trimmed.len() < 4 {
        return Err(Psk31Error::Cat(format!(
            "Invalid mode response: '{response}'"
        )));
    }
    let code = &trimmed[3..4];
    MODE_TABLE
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, name)| CatResponse::Mode(name.to_string()))
        .ok_or_else(|| Psk31Error::Cat(format!("Unknown mode code: '{code}'")))
}

/// Parse `"PC050;"` → `TxPower(50)`
fn parse_tx_power(response: &str) -> Psk31Result<CatResponse> {
    let trimmed = response.trim().trim_end_matches(';');
    if !trimmed.starts_with("PC") || trimmed.len() < 5 {
        return Err(Psk31Error::Cat(format!(
            "Invalid TX power response: '{response}'"
        )));
    }
    let digits = &trimmed[2..5];
    let watts = digits
        .parse::<u32>()
        .map_err(|e| Psk31Error::Cat(format!("Failed to parse TX power '{digits}': {e}")))?;
    Ok(CatResponse::TxPower(watts))
}

/// For commands where the radio only returns `";"` (or empty Ack).
fn expect_ack(response: &str, cmd: &CatCommand) -> Psk31Result<CatResponse> {
    let trimmed = response.trim();
    if trimmed == ";" || trimmed.is_empty() {
        Ok(CatResponse::Ack)
    } else {
        Err(Psk31Error::Cat(format!(
            "Expected Ack (';') for {cmd:?}, got: '{response}'"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use CatCommand::*;

    // --- NAK ---

    #[test]
    fn nak_returns_err() {
        assert!(decode("?", &GetFrequencyA).is_err());
        assert!(decode("?;", &GetFrequencyA).is_err());
        assert!(decode("?", &PttOn).is_err());
    }

    // --- GetFrequencyA ---

    #[test]
    fn decode_frequency_20m() {
        assert_eq!(
            decode("FA00014070000;", &GetFrequencyA).unwrap(),
            CatResponse::FrequencyHz(14_070_000)
        );
    }

    #[test]
    fn decode_frequency_40m() {
        assert_eq!(
            decode("FA00007074000;", &GetFrequencyA).unwrap(),
            CatResponse::FrequencyHz(7_074_000)
        );
    }

    #[test]
    fn decode_frequency_invalid_prefix() {
        assert!(decode("FB00014070000;", &GetFrequencyA).is_err());
    }

    #[test]
    fn decode_frequency_too_short() {
        assert!(decode("FA123;", &GetFrequencyA).is_err());
    }

    // --- SetFrequencyA ---

    #[test]
    fn decode_set_frequency_ack() {
        assert_eq!(
            decode(";", &SetFrequencyA(14_070_000)).unwrap(),
            CatResponse::Ack
        );
    }

    // --- GetMode ---

    #[test]
    fn decode_mode_data_usb() {
        assert_eq!(
            decode("MD0C;", &GetMode).unwrap(),
            CatResponse::Mode("DATA-USB".into())
        );
    }

    #[test]
    fn decode_mode_usb() {
        assert_eq!(
            decode("MD02;", &GetMode).unwrap(),
            CatResponse::Mode("USB".into())
        );
    }

    #[test]
    fn decode_mode_lsb() {
        assert_eq!(
            decode("MD01;", &GetMode).unwrap(),
            CatResponse::Mode("LSB".into())
        );
    }

    #[test]
    fn decode_mode_unknown_code() {
        assert!(decode("MD0Z;", &GetMode).is_err());
    }

    #[test]
    fn decode_mode_too_short() {
        assert!(decode("MD;", &GetMode).is_err());
    }

    // --- SetMode ---

    #[test]
    fn decode_set_mode_ack() {
        assert_eq!(
            decode(";", &SetMode("DATA-USB".into())).unwrap(),
            CatResponse::Ack
        );
    }

    // --- PTT ---

    #[test]
    fn decode_ptt_on_ack() {
        assert_eq!(decode(";", &PttOn).unwrap(), CatResponse::Ack);
    }

    #[test]
    fn decode_ptt_off_ack() {
        assert_eq!(decode(";", &PttOff).unwrap(), CatResponse::Ack);
    }

    // --- GetTxPower ---

    #[test]
    fn decode_tx_power_50w() {
        assert_eq!(
            decode("PC050;", &GetTxPower).unwrap(),
            CatResponse::TxPower(50)
        );
    }

    #[test]
    fn decode_tx_power_100w() {
        assert_eq!(
            decode("PC100;", &GetTxPower).unwrap(),
            CatResponse::TxPower(100)
        );
    }

    #[test]
    fn decode_tx_power_invalid() {
        assert!(decode("PCXXX;", &GetTxPower).is_err());
        assert!(decode("PC;", &GetTxPower).is_err());
    }

    // --- SetTxPower ---

    #[test]
    fn decode_set_tx_power_ack() {
        assert_eq!(decode(";", &SetTxPower(25)).unwrap(), CatResponse::Ack);
    }

    // --- Mode roundtrip ---

    #[test]
    fn decode_all_modes_roundtrip() {
        for (code, name) in MODE_TABLE {
            let response = format!("MD0{code};");
            let decoded = decode(&response, &GetMode).unwrap();
            assert_eq!(
                decoded,
                CatResponse::Mode(name.to_string()),
                "Roundtrip failed for mode code '{code}'"
            );
        }
    }
}
