use anyhow::{Context, Result};
use colored::Colorize;
use globset::{Glob, GlobSet, GlobSetBuilder};
use sha2::{Digest, Sha256};
use std::fs::{self, File, read_to_string};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, IntoIter as WalkDirIter, WalkDir};
use zip::write::{FileOptions, SimpleFileOptions};
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub struct FilteredWalker {
    root: PathBuf,
    walker: WalkDirIter,
    builder: GlobSetBuilder,
    ignore_set: Option<GlobSet>,
}

impl FilteredWalker {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            walker: WalkDir::new(root).into_iter(),
            builder: GlobSetBuilder::new(),
            ignore_set: None,
        }
    }

    pub fn add_ignore_file(&mut self, filename: &Path) -> &mut Self {
        let full_path = self.root.join(filename);
        if let Ok(content) = read_to_string(full_path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                let pattern = if line.ends_with('/') {
                    format!("**/{}/**", line.trim_end_matches('/'))
                } else {
                    format!("**/{}", line)
                };

                if let Ok(glob) = Glob::new(&pattern) {
                    self.builder.add(glob);
                }
            }
        }
        self
    }

    #[allow(unused)]
    pub fn add_pattern(&mut self, pattern: &str) -> &mut Self {
        if let Ok(glob) = Glob::new(pattern) {
            self.builder.add(glob);
        }
        self
    }
}

impl Iterator for FilteredWalker {
    type Item = DirEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ignore_set.is_none() {
            self.ignore_set = Some(self.builder.build().expect("Error compilando patrones"));
        }

        let ignore_set = self.ignore_set.as_ref().unwrap();

        loop {
            let entry = self.walker.next()?;

            match entry {
                Ok(e) => {
                    if e.depth() > 0 && ignore_set.is_match(e.path()) {
                        if e.file_type().is_dir() {
                            self.walker.skip_current_dir();
                        }
                        continue;
                    }
                    return Some(e);
                },
                Err(_) => continue,
            }
        }
    }
}

pub fn create_zip(dir: &Path) -> Result<(Vec<u8>, Vec<String>, String)> {
    let cursor = std::io::Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(cursor);
    let mut files = Vec::new();

    let options: SimpleFileOptions = FileOptions::default().compression_method(CompressionMethod::Deflated);
    let vkignore = dir.join(".vkignore");
    let gitignore = dir.join(".gitignore");
    let mut walker = FilteredWalker::new(dir);

    if vkignore.exists() {
        walker.add_ignore_file(&vkignore);
    }

    if gitignore.exists() {
        walker.add_ignore_file(&gitignore);
    }

    println!("\nFiles included:");

    for entry in walker {
        let path = entry.path();

        if path.is_file() {
            let name = path.strip_prefix(dir)?.to_str().context("invalid path")?;

            zip.start_file(name, options)?;
            let mut file = File::open(path)?;
            std::io::copy(&mut file, &mut zip)?;

            files.push(name.to_string());

            println!("	{} {}", "✓".green(), name);
        }
    }

    if files.is_empty() {
        return Err(anyhow::anyhow!("No files to include in the package"));
    }

    let cursor = zip.finish()?;
    let buffer = cursor.into_inner();

    let mut hasher = Sha256::new();
    hasher.update(&buffer);
    let checksum = hex::encode(hasher.finalize());

    Ok((buffer, files, checksum))
}

pub fn extract_zip(data: &[u8], dest_dir: &Path) -> Result<()> {
    let cursor = std::io::Cursor::new(data);
    let mut archive = ZipArchive::new(cursor)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => dest_dir.join(path),
            None => continue,
        };

        if file.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}

pub fn parse_package(spec: &str) -> (String, Option<String>) {
    match spec.split_once('@') {
        Some((id, version)) => (id.to_string(), Some(version.to_string())),
        None => (spec.to_string(), None),
    }
}

pub fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;

    if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
