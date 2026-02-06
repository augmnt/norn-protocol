use dialoguer::{Confirm, Password};

use super::error::WalletError;

/// Prompt the user for a password (hidden input).
pub fn prompt_password(prompt: &str) -> Result<String, WalletError> {
    Password::new()
        .with_prompt(prompt)
        .interact()
        .map_err(|e| WalletError::IoError(std::io::Error::other(e)))
}

/// Prompt the user for a new password with confirmation.
pub fn prompt_new_password() -> Result<String, WalletError> {
    Password::new()
        .with_prompt("Enter password")
        .with_confirmation("Confirm password", "Passwords do not match")
        .interact()
        .map_err(|e| WalletError::IoError(std::io::Error::other(e)))
}

/// Ask the user to confirm an action.
pub fn confirm(prompt: &str) -> Result<bool, WalletError> {
    Confirm::new()
        .with_prompt(prompt)
        .default(false)
        .interact()
        .map_err(|e| WalletError::IoError(std::io::Error::other(e)))
}
