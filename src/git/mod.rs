pub mod ephemeral;
pub mod operations;
pub mod repo_layout;
pub mod safety;

pub use ephemeral::EphemeralRepoGuard;
pub use operations::{
    checkout_branch, checkout_or_create_branch, clone_repo, commit_and_push, commit_merge,
    create_branch, fetch_branch, init_repo, list_branches, merge_branch, merge_dry_run,
    pull_latest, run_mergetool, set_remote_origin,
};
pub use repo_layout::read_app_files;
pub use safety::{check_file_safety, confirm_operation};
