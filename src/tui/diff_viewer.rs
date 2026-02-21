use crate::error::Result;

pub fn show_diff_tui(diff: &str) -> Result<bool> {
    // TODO: Implement TUI diff viewer
    println!("{}", diff);
    Ok(true)
}
