pub mod comments;
pub mod format;
pub mod sections;

pub use format::{FileFormat, detect_format};
pub use sections::{extract_syncable_content, merge_synced_content, detect_comment_syntax};
