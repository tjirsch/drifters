pub mod ephemeral;
pub mod operations;
pub mod safety;

pub use ephemeral::{cleanup_ephemeral_repo, setup_ephemeral_repo, EphemeralRepoGuard};
pub use operations::{clone_repo, commit_and_push, get_remote_url, init_repo, open_repo, pull_latest};
pub use safety::{check_file_safety, confirm_operation};
