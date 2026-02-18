use anyhow::Result;
use std::path::Path;

use crate::manifest::MANIFEST_FILENAME;

/// This package contains the pre-run command for the vayload-kit commands;
///
pub fn ensure_manifest_exists() -> Result<()> {
    let manifest_path = Path::new(MANIFEST_FILENAME);

    if !manifest_path.exists() {
        anyhow::bail!(
            "No {} found in the current directory.\n\
             This command must be run inside a Vayload project.\n\
             Run `vk init` to create a new project.",
            MANIFEST_FILENAME
        );
    }

    Ok(())
}
