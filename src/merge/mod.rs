pub mod apply_rules;
pub mod conflict;
pub mod three_way;

pub use three_way::merge_configs;
pub use conflict::ConflictResolution;
