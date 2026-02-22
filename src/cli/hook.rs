use crate::error::Result;

pub fn generate_hook() -> Result<()> {
    // Output shell code that will be eval'd
    println!(
        r#"# Drifters auto-sync hook
# This runs in the background when you start a new shell

drifters_auto_sync() {{
    # Run in background, suppress all output
    (drifters pull-app --yolo >/dev/null 2>&1 &)
}}

# Run on shell startup
drifters_auto_sync
"#
    );

    Ok(())
}
