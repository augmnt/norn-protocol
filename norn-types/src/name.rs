use crate::constants::ONE_NORN;
use crate::error::NornError;
use crate::primitives::Amount;

/// Fee for registering a name (1 NORN, burned).
pub const NAME_REGISTRATION_FEE: Amount = ONE_NORN;

/// Validate a name: lowercase alphanumeric + hyphens, 3-32 chars, no leading/trailing hyphens.
pub fn validate_name(name: &str) -> Result<(), NornError> {
    if name.len() < 3 || name.len() > 32 {
        return Err(NornError::InvalidName(format!(
            "name must be 3-32 characters, got {}",
            name.len()
        )));
    }
    if name.starts_with('-') || name.ends_with('-') {
        return Err(NornError::InvalidName(
            "name must not start or end with a hyphen".to_string(),
        ));
    }
    for c in name.chars() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '-' {
            return Err(NornError::InvalidName(format!(
                "name must be lowercase alphanumeric or hyphens, found '{}'",
                c
            )));
        }
    }
    Ok(())
}
