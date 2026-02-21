pub mod ephemeral;
pub mod operations;
pub mod safety;

pub use ephemeral::EphemeralRepoGuard;
pub use operations::{clone_repo, commit_and_push, init_repo, pull_latest};
pub use safety::{check_file_safety, confirm_operation};
