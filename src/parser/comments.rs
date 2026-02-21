use super::format::FileFormat;

pub fn get_comment_syntax(format: &FileFormat) -> &'static str {
    match format {
        FileFormat::Json => "//",
        FileFormat::Yaml => "#",
        FileFormat::Toml => "#",
        FileFormat::Shell => "#",
        FileFormat::Text => "#",
    }
}
