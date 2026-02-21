use crate::error::Result;

pub enum ConflictResolution {
    KeepLocal,
    KeepRemote,
    Manual(String),
}

pub fn resolve_conflict(local: &str, remote: &str) -> Result<ConflictResolution> {
    // TODO: Implement interactive conflict resolution
    Ok(ConflictResolution::KeepLocal)
}
