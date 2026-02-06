use console::Style;
use norn_types::constants::{NORN_DECIMALS, ONE_NORN};
use norn_types::primitives::{Address, Amount, Hash, PublicKey, TokenId, NATIVE_TOKEN_ID};

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

/// Format an Amount into a human-readable string like "1,234.567890120000".
pub fn format_amount(amount: Amount) -> String {
    let decimals = NORN_DECIMALS as usize;
    let whole = amount / ONE_NORN;
    let frac = amount % ONE_NORN;

    let whole_str = format_with_commas(whole);
    let frac_str = format!("{:0>width$}", frac, width = decimals);

    format!("{}.{}", whole_str, frac_str)
}

/// Format amount with token symbol.
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

/// Parse a human-readable amount string (e.g. "10.5") into an Amount.
pub fn parse_amount(s: &str) -> Result<Amount, WalletError> {
    let s = s.replace(',', "");
    let parts: Vec<&str> = s.split('.').collect();

    match parts.len() {
        1 => {
            let whole: u128 = parts[0]
                .parse()
                .map_err(|_| WalletError::InvalidAmount(s.clone()))?;
            Ok(whole * ONE_NORN)
        }
        2 => {
            let whole: u128 = if parts[0].is_empty() {
                0
            } else {
                parts[0]
                    .parse()
                    .map_err(|_| WalletError::InvalidAmount(s.clone()))?
            };
            let decimals = NORN_DECIMALS as usize;
            let frac_str = if parts[1].len() > decimals {
                &parts[1][..decimals]
            } else {
                parts[1]
            };
            let padded = format!("{:0<width$}", frac_str, width = decimals);
            let frac: u128 = padded
                .parse()
                .map_err(|_| WalletError::InvalidAmount(s.clone()))?;
            Ok(whole * ONE_NORN + frac)
        }
        _ => Err(WalletError::InvalidAmount(s)),
    }
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

/// Format an Address in truncated form: 0xab12...ef34
#[allow(dead_code)]
pub fn format_address_short(addr: &Address) -> String {
    let full = hex::encode(addr);
    format!("0x{}...{}", &full[..4], &full[full.len() - 4..])
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

/// Format a Hash as hex.
#[allow(dead_code)]
pub fn format_hash(hash: &Hash) -> String {
    hex::encode(hash)
}

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

/// Print an informational line.
#[allow(dead_code)]
pub fn print_info(label: &str, value: &str) {
    println!(
        "  {}: {}",
        style_bold().apply_to(label),
        style_info().apply_to(value)
    );
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
    fn test_format_address_short() {
        let addr = [0xab; 20];
        let short = format_address_short(&addr);
        assert!(short.starts_with("0x"));
        assert!(short.contains("..."));
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
}
