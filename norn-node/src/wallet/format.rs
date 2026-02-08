use console::Style;
use norn_types::constants::NORN_DECIMALS;
use norn_types::primitives::{Address, Amount, PublicKey, TokenId, NATIVE_TOKEN_ID};

use super::error::WalletError;

// ── Styles ──────────────────────────────────────────────────────────────────

pub fn style_success() -> Style {
    Style::new().green()
}

pub fn style_error() -> Style {
    Style::new().red()
}

pub fn style_warn() -> Style {
    Style::new().yellow()
}

pub fn style_info() -> Style {
    Style::new().cyan()
}

pub fn style_bold() -> Style {
    Style::new().bold()
}

pub fn style_dim() -> Style {
    Style::new().dim()
}

// ── Amount formatting ───────────────────────────────────────────────────────

/// Format an Amount using a specific number of decimal places.
/// For example, `format_token_amount(2_110_000_00, 8)` => `"2.11000000"`.
pub fn format_token_amount(amount: Amount, decimals: u8) -> String {
    if decimals == 0 {
        return format_with_commas(amount);
    }
    let decimal_places = decimals as usize;
    let divisor: u128 = 10u128.pow(decimals as u32);
    let whole = amount / divisor;
    let frac = amount % divisor;

    let whole_str = format_with_commas(whole);
    let frac_str = format!("{:0>width$}", frac, width = decimal_places);

    format!("{}.{}", whole_str, frac_str)
}

/// Format a token amount with its symbol name.
pub fn format_token_amount_with_name(amount: Amount, decimals: u8, symbol: &str) -> String {
    format!("{} {}", format_token_amount(amount, decimals), symbol)
}

/// Format a native NORN Amount into a human-readable string like "1,234.567890120000".
pub fn format_amount(amount: Amount) -> String {
    format_token_amount(amount, NORN_DECIMALS as u8)
}

/// Format amount with token symbol (native NORN only — uses NORN decimals).
pub fn format_amount_with_symbol(amount: Amount, token_id: &TokenId) -> String {
    let formatted = format_amount(amount);
    if *token_id == NATIVE_TOKEN_ID {
        format!("{} NORN", formatted)
    } else {
        format!(
            "{} (token:{})",
            formatted,
            truncate_hex(&hex::encode(token_id), 8)
        )
    }
}

/// Parse a human-readable amount string using a specific number of decimal places.
/// For example, `parse_token_amount("10.5", 8)` => `1_050_000_000`.
pub fn parse_token_amount(s: &str, decimals: u8) -> Result<Amount, WalletError> {
    let s = s.replace([',', '_'], "");

    if decimals == 0 {
        return s.parse().map_err(|_| WalletError::InvalidAmount(s.clone()));
    }

    let decimal_places = decimals as usize;
    let divisor: u128 = 10u128.pow(decimals as u32);
    let parts: Vec<&str> = s.split('.').collect();

    match parts.len() {
        1 => {
            let whole: u128 = parts[0]
                .parse()
                .map_err(|_| WalletError::InvalidAmount(s.clone()))?;
            Ok(whole * divisor)
        }
        2 => {
            let whole: u128 = if parts[0].is_empty() {
                0
            } else {
                parts[0]
                    .parse()
                    .map_err(|_| WalletError::InvalidAmount(s.clone()))?
            };
            let frac_str = if parts[1].len() > decimal_places {
                &parts[1][..decimal_places]
            } else {
                parts[1]
            };
            let padded = format!("{:0<width$}", frac_str, width = decimal_places);
            let frac: u128 = padded
                .parse()
                .map_err(|_| WalletError::InvalidAmount(s.clone()))?;
            Ok(whole * divisor + frac)
        }
        _ => Err(WalletError::InvalidAmount(s)),
    }
}

/// Parse a human-readable amount string (e.g. "10.5") into a native NORN Amount.
#[allow(dead_code)]
pub fn parse_amount(s: &str) -> Result<Amount, WalletError> {
    parse_token_amount(s, NORN_DECIMALS as u8)
}

fn format_with_commas(n: u128) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

// ── Address formatting ──────────────────────────────────────────────────────

/// Format an Address as a 0x-prefixed hex string.
pub fn format_address(addr: &Address) -> String {
    format!("0x{}", hex::encode(addr))
}

/// Parse a hex address string (with or without 0x prefix) into an Address.
pub fn parse_address(s: &str) -> Result<Address, WalletError> {
    let hex_str = s.strip_prefix("0x").unwrap_or(s);
    if hex_str.len() != 40 {
        return Err(WalletError::InvalidAddress(format!(
            "expected 40 hex chars, got {}",
            hex_str.len()
        )));
    }
    let bytes = hex::decode(hex_str)
        .map_err(|e| WalletError::InvalidAddress(format!("invalid hex: {}", e)))?;
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&bytes);
    Ok(addr)
}

/// Parse a hex token ID string into a TokenId.
/// Note: For commands that need symbol-based resolution, use `resolve_token()` instead.
#[allow(dead_code)]
pub fn parse_token_id(s: &str) -> Result<TokenId, WalletError> {
    let hex_str = s.strip_prefix("0x").unwrap_or(s);
    if hex_str == "NORN" || hex_str == "norn" || hex_str == "native" {
        return Ok(NATIVE_TOKEN_ID);
    }
    if hex_str.len() != 64 {
        return Err(WalletError::InvalidAddress(format!(
            "token ID: expected 64 hex chars, got {}",
            hex_str.len()
        )));
    }
    let bytes = hex::decode(hex_str)
        .map_err(|e| WalletError::InvalidAddress(format!("invalid token hex: {}", e)))?;
    let mut id = [0u8; 32];
    id.copy_from_slice(&bytes);
    Ok(id)
}

// ── Hash / pubkey formatting ────────────────────────────────────────────────

/// Format a PublicKey as hex.
pub fn format_pubkey(pk: &PublicKey) -> String {
    hex::encode(pk)
}

fn truncate_hex(hex: &str, len: usize) -> String {
    if hex.len() <= len * 2 {
        hex.to_string()
    } else {
        format!("{}...{}", &hex[..len], &hex[hex.len() - len..])
    }
}

/// Truncate a hex string (with or without 0x prefix) to show first/last `half_len` chars.
/// E.g. `truncate_hex_string("0xabcdef123456", 5)` => `"0xabcde...23456"`
pub fn truncate_hex_string(s: &str, half_len: usize) -> String {
    let (prefix, hex) = if let Some(stripped) = s.strip_prefix("0x") {
        ("0x", stripped)
    } else {
        ("", s)
    };
    if hex.len() <= half_len * 2 {
        s.to_string()
    } else {
        format!(
            "{}{}...{}",
            prefix,
            &hex[..half_len],
            &hex[hex.len() - half_len..]
        )
    }
}

// ── Display helpers ─────────────────────────────────────────────────────────

/// Print a success message.
pub fn print_success(msg: &str) {
    println!("  {} {}", style_success().apply_to("✓"), msg);
}

/// Print an error message with a hint.
pub fn print_error(msg: &str, hint: Option<&str>) {
    eprintln!("  {} {}", style_error().apply_to("Error:"), msg);
    if let Some(h) = hint {
        eprintln!(
            "  {} {}",
            style_dim().apply_to("Hint:"),
            style_dim().apply_to(h)
        );
    }
}

/// Print a divider.
pub fn print_divider() {
    println!(
        "  {}",
        style_dim().apply_to("────────────────────────────────")
    );
}

/// Print the mnemonic in a warning box.
pub fn print_mnemonic_box(words: &[&str]) {
    let warn = style_warn();
    let bold = style_bold();

    println!();
    println!(
        "  {}",
        warn.apply_to("╔══════════════════════════════════════════════════════════════╗")
    );
    println!(
        "  {}",
        warn.apply_to("║  IMPORTANT: Write down these 24 words.                      ║")
    );
    println!(
        "  {}",
        warn.apply_to("║  They are the ONLY way to recover your wallet.              ║")
    );
    println!(
        "  {}",
        warn.apply_to("║  Store them safely offline.                                 ║")
    );
    println!(
        "  {}",
        warn.apply_to("╠══════════════════════════════════════════════════════════════╣")
    );
    println!(
        "  {}",
        warn.apply_to("║                                                              ║")
    );

    // Print words in rows of 4
    for row in words.chunks(4) {
        let mut line = String::from("  ║   ");
        for (i, word) in row.iter().enumerate() {
            let idx = words
                .iter()
                .position(|w| std::ptr::eq(*w, *word))
                .unwrap_or(0)
                + 1;
            let entry = format!("{:>2}. {:<12}", idx, word);
            line.push_str(&entry);
            if i < row.len() - 1 {
                line.push_str("  ");
            }
        }
        // Pad to box width
        let content_width: usize = 62; // inner width of the box
        let visible_len = line.len() - 2; // subtract "  " prefix for box alignment
        let padding = content_width.saturating_sub(visible_len);
        line.push_str(&" ".repeat(padding));
        line.push('║');
        println!("{}", bold.apply_to(&line));
    }

    println!(
        "  {}",
        warn.apply_to("║                                                              ║")
    );
    println!(
        "  {}",
        warn.apply_to("╚══════════════════════════════════════════════════════════════╝")
    );
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_types::constants::ONE_NORN;

    #[test]
    fn test_format_amount_zero() {
        assert_eq!(format_amount(0), "0.000000000000");
    }

    #[test]
    fn test_format_amount_one_norn() {
        assert_eq!(format_amount(ONE_NORN), "1.000000000000");
    }

    #[test]
    fn test_format_amount_fractional() {
        assert_eq!(format_amount(ONE_NORN / 2), "0.500000000000");
    }

    #[test]
    fn test_format_amount_large() {
        assert_eq!(
            format_amount(1_234 * ONE_NORN + 567890120000),
            "1,234.567890120000"
        );
    }

    #[test]
    fn test_parse_amount_whole() {
        assert_eq!(parse_amount("10").unwrap(), 10 * ONE_NORN);
    }

    #[test]
    fn test_parse_amount_decimal() {
        assert_eq!(parse_amount("10.5").unwrap(), 10 * ONE_NORN + ONE_NORN / 2);
    }

    #[test]
    fn test_parse_amount_with_commas() {
        assert_eq!(parse_amount("1,000").unwrap(), 1000 * ONE_NORN);
    }

    #[test]
    fn test_parse_amount_roundtrip() {
        let amount = 1234567890123u128;
        let formatted = format_amount(amount);
        let parsed = parse_amount(&formatted).unwrap();
        assert_eq!(parsed, amount);
    }

    #[test]
    fn test_format_address() {
        let addr = [0xab; 20];
        let formatted = format_address(&addr);
        assert!(formatted.starts_with("0x"));
        assert_eq!(formatted.len(), 42);
    }

    #[test]
    fn test_parse_address_with_prefix() {
        let addr = [0xab; 20];
        let hex = format!("0x{}", hex::encode(addr));
        let parsed = parse_address(&hex).unwrap();
        assert_eq!(parsed, addr);
    }

    #[test]
    fn test_parse_address_without_prefix() {
        let addr = [0xab; 20];
        let hex = hex::encode(addr);
        let parsed = parse_address(&hex).unwrap();
        assert_eq!(parsed, addr);
    }

    #[test]
    fn test_parse_address_invalid_length() {
        assert!(parse_address("0xdeadbeef").is_err());
    }

    #[test]
    fn test_parse_token_id_native() {
        assert_eq!(parse_token_id("NORN").unwrap(), NATIVE_TOKEN_ID);
        assert_eq!(parse_token_id("norn").unwrap(), NATIVE_TOKEN_ID);
        assert_eq!(parse_token_id("native").unwrap(), NATIVE_TOKEN_ID);
    }

    // ── Token-amount (custom decimals) tests ──────────────────────────────

    #[test]
    fn test_format_token_amount_8_decimals() {
        // 211_000_000 raw with 8 decimals = 2.11
        assert_eq!(format_token_amount(211_000_000, 8), "2.11000000");
    }

    #[test]
    fn test_format_token_amount_zero_decimals() {
        assert_eq!(format_token_amount(42, 0), "42");
        assert_eq!(format_token_amount(1_234_567, 0), "1,234,567");
    }

    #[test]
    fn test_format_token_amount_6_decimals() {
        // 1_500_000 raw with 6 decimals = 1.5
        assert_eq!(format_token_amount(1_500_000, 6), "1.500000");
    }

    #[test]
    fn test_parse_token_amount_8_decimals() {
        assert_eq!(parse_token_amount("2.11", 8).unwrap(), 211_000_000);
        assert_eq!(parse_token_amount("100", 8).unwrap(), 100 * 10u128.pow(8));
    }

    #[test]
    fn test_parse_token_amount_zero_decimals() {
        assert_eq!(parse_token_amount("42", 0).unwrap(), 42);
    }

    #[test]
    fn test_format_parse_token_roundtrip_8() {
        let raw = 123_456_789u128;
        let formatted = format_token_amount(raw, 8);
        let parsed = parse_token_amount(&formatted, 8).unwrap();
        assert_eq!(parsed, raw);
    }

    #[test]
    fn test_format_parse_token_roundtrip_6() {
        let raw = 1_234_567u128;
        let formatted = format_token_amount(raw, 6);
        let parsed = parse_token_amount(&formatted, 6).unwrap();
        assert_eq!(parsed, raw);
    }

    #[test]
    fn test_norn_delegates_to_token_amount() {
        // format_amount should produce same result as format_token_amount with 12 decimals
        let amount = 1234567890123u128;
        assert_eq!(format_amount(amount), format_token_amount(amount, 12));
    }
}
