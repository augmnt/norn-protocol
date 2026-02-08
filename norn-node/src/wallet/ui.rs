use comfy_table::{presets, CellAlignment, ContentArrangement, Table};

/// Data table for lists (tokens, validators, history, wallets).
/// UTF8_FULL preset with header separator, dynamic width.
pub fn data_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(headers);
    table
}

/// Key-value info card (token-info, node-info, fees, block, whoami).
/// No header, no outer borders -- just clean aligned rows.
pub fn info_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(presets::NOTHING)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table
}

/// Right-aligned cell (for amounts / numbers).
pub fn cell_right(content: impl ToString) -> comfy_table::Cell {
    comfy_table::Cell::new(content).set_alignment(CellAlignment::Right)
}

/// Print table with 2-space left indent to match existing CLI aesthetic.
pub fn print_table(table: &Table) {
    for line in table.lines() {
        println!("  {}", line);
    }
}
