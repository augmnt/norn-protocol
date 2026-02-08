use comfy_table::{presets, Attribute, CellAlignment, Color, ContentArrangement, Table};

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

/// Plain cell.
pub fn cell(content: impl ToString) -> comfy_table::Cell {
    comfy_table::Cell::new(content)
}

/// Green cell (success / received).
pub fn cell_green(content: impl ToString) -> comfy_table::Cell {
    comfy_table::Cell::new(content).fg(Color::Green)
}

/// Yellow cell (warning / sent).
pub fn cell_yellow(content: impl ToString) -> comfy_table::Cell {
    comfy_table::Cell::new(content).fg(Color::Yellow)
}

/// Cyan cell (info / emphasis).
pub fn cell_cyan(content: impl ToString) -> comfy_table::Cell {
    comfy_table::Cell::new(content).fg(Color::Cyan)
}

/// Bold cell.
pub fn cell_bold(content: impl ToString) -> comfy_table::Cell {
    comfy_table::Cell::new(content).add_attribute(Attribute::Bold)
}

/// Dim cell.
pub fn cell_dim(content: impl ToString) -> comfy_table::Cell {
    comfy_table::Cell::new(content).add_attribute(Attribute::Dim)
}

/// Print table with 2-space left indent to match existing CLI aesthetic.
pub fn print_table(table: &Table) {
    for line in table.lines() {
        println!("  {}", line);
    }
}
