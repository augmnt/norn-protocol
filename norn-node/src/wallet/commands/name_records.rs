use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{print_error, style_bold, style_dim, style_info};
use crate::wallet::rpc_client::RpcClient;
use crate::wallet::ui::{data_table, print_table};

pub async fn run(name: &str, json: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    // Verify name exists.
    if rpc.resolve_name(name).await?.is_none() {
        print_error(&format!("name '{}' is not registered", name), None);
        return Ok(());
    }

    let records = rpc.get_name_records(name).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&records)?);
    } else {
        println!();
        println!(
            "  {} for {}",
            style_bold().apply_to("NNS Records"),
            style_info().apply_to(name)
        );

        if records.is_empty() {
            println!(
                "  {}",
                style_dim()
                    .apply_to("No records set. Use `norn wallet set-name-record` to add one.")
            );
        } else {
            let mut table = data_table(&["Key", "Value"]);
            let mut keys: Vec<&String> = records.keys().collect();
            keys.sort();
            for key in keys {
                table.add_row(vec![key.as_str(), records[key].as_str()]);
            }
            print_table(&table);
            println!(
                "  {}",
                style_dim().apply_to(format!("{} record(s)", records.len()))
            );
        }
        println!();
    }

    Ok(())
}
