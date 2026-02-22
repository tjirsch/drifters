pub mod ephemeral;
pub mod operations;
pub mod repo_layout;
pub mod safety;

pub use ephemeral::EphemeralRepoGuard;
pub use operations::{clone_repo, commit_and_push, init_repo, pull_latest, set_remote_origin};
pub use repo_layout::collect_machine_versions;
pub use safety::{check_file_safety, confirm_operation};
