use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::style_dim;

pub async fn run(_limit: usize, _json: bool) -> Result<(), WalletError> {
    let _config = WalletConfig::load()?;

    // Transaction history requires indexing which is not yet implemented.
    // This is a placeholder that will be extended when the node supports
    // transaction indexing/querying.
    println!();
    println!(
        "  {}",
        style_dim().apply_to("Transaction history is not yet available.")
    );
    println!(
        "  {}",
        style_dim().apply_to("This feature requires transaction indexing on the node.")
    );
    println!();

    Ok(())
}
