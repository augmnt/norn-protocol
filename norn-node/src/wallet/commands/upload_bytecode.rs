use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{print_error, print_success, style_bold, style_dim};
use crate::wallet::rpc_client::RpcClient;

pub async fn run(loom_id: &str, path: &str, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    // Read the .wasm file from disk.
    let bytecode = std::fs::read(path).map_err(|e| {
        WalletError::Other(format!("failed to read bytecode file '{}': {}", path, e))
    })?;

    if bytecode.is_empty() {
        return Err(WalletError::Other("bytecode file is empty".to_string()));
    }

    println!();
    println!("  {}", style_bold().apply_to("Upload Bytecode"));
    println!("  Loom ID: {}", style_dim().apply_to(loom_id));
    println!("  File:    {}", style_dim().apply_to(path));
    println!(
        "  Size:    {} bytes",
        style_dim().apply_to(bytecode.len().to_string())
    );
    println!();

    let bytecode_hex = hex::encode(&bytecode);

    let result = rpc.upload_loom_bytecode(loom_id, &bytecode_hex).await?;

    if result.success {
        print_success("Bytecode uploaded and initialized!");
    } else {
        print_error(
            &format!(
                "Bytecode upload failed: {}",
                result.reason.unwrap_or_else(|| "unknown".to_string())
            ),
            None,
        );
    }
    println!();

    Ok(())
}
