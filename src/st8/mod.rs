pub mod st8_common;
pub mod templates;

pub use st8_common::{St8Config, VersionInfo, detect_project_files, ProjectFile, ProjectFileType, update_version_file};
pub use templates::{TemplateManager, TemplateConfig};