use anyhow::{Context, Result};
use colored::Colorize;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::fs::{File, read_to_string};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};

use crate::manifest::VKIGNORE_FILENAME;

use globset::{Glob, GlobSet, GlobSetBuilder};
use walkdir::{DirEntry, IntoIter as WalkDirIter, WalkDir};

pub struct FilteredWalker {
    root: PathBuf,
    walker: WalkDirIter,
    builder: GlobSetBuilder,
    ignore_set: Option<GlobSet>,
}

impl FilteredWalker {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        let mut builder = GlobSetBuilder::new();

        // core ignore patterns
        let default_ignores = ["**/.git/**", "**/.svn/**", "**/.hg/**", "**/.vk/**", "**/.vkcache/**"];

        for pattern in default_ignores.iter() {
            builder.add(Glob::new(pattern).expect("fails on default ignore pattern"));
        }

        Self {
            root: root.as_ref().to_path_buf(),
            walker: WalkDir::new(&root).into_iter(),
            builder,
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
                    let rel_path = e.path().strip_prefix(&self.root).unwrap_or(e.path());

                    if e.depth() > 0 && ignore_set.is_match(rel_path) {
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

// Maximum allowed ZIP size for this implementation is 25 MB.
// (Future: could be increased up to 250 MB for larger packages)
const MAX_LIMIT_SIZE: usize = 25 * 1024 * 1024; // 25MB

pub struct PluginPackager;

#[allow(unused)]
impl PluginPackager {
    pub fn new() -> Self {
        Self
    }

    /// Create a tar.gz file from the input directory.
    /// Returns the path of the created tar.gz file.
    pub fn pack(&self, input_dir: &Path, output_path: &Path, verbose: bool) -> Result<PathBuf> {
        if verbose {
            println!(
                "\n{} Preparing tar.gz package from: {}",
                "📦".bold().blue(),
                input_dir.display().to_string().bright_black()
            );
            println!("{}", "-".repeat(80));
            println!("{:<2} {:<60} {:>15}", "", "File", "Size");
            println!("{}", "-".repeat(80));
        }

        let file =
            File::create(output_path).with_context(|| format!("Unable to create temp directory: {:?}", output_path))?;

        self.pack_to_writer(file, input_dir, verbose)?;

        if verbose {
            println!("{}", "-".repeat(80));
            println!(
                "{} Package successfully created at: {}",
                "✨".green(),
                output_path.display()
            );
        }

        Ok(output_path.to_path_buf())
    }

    pub fn pack_to_writer<W: Write>(&self, writer: W, input_dir: &Path, verbose: bool) -> Result<()> {
        let enc = GzEncoder::new(writer, Compression::default());
        let mut tar = Builder::new(enc);

        let mut walker = FilteredWalker::new(input_dir);

        let vkignore = input_dir.join(VKIGNORE_FILENAME);
        if vkignore.exists() {
            walker.add_ignore_file(&vkignore);
        }
        let gitignore = input_dir.join(".gitignore");
        if gitignore.exists() {
            walker.add_ignore_file(&gitignore);
        }

        let mut total_size: usize = 0;

        for entry in walker {
            let path = entry.path();
            let relative_path = path.strip_prefix(input_dir).context("Error calculando ruta relativa del archivo")?;

            if path.is_file() {
                let mut f = File::open(path)?;
                let size = path.metadata().map(|m| m.len()).unwrap_or(0);

                if verbose {
                    println!(
                        "{} {:<60} {:>15}",
                        "✓".green(),
                        relative_path.display(),
                        format_bytes(size as usize).bright_black()
                    );
                }

                total_size += size as usize;
                if total_size > MAX_LIMIT_SIZE {
                    return Err(anyhow::anyhow!("Package size exceeds maximum limit"));
                }

                tar.append_file(relative_path, &mut f)?;
            } else if path.is_dir() && !relative_path.as_os_str().is_empty() {
                tar.append_dir(relative_path, path)?;
            }
        }

        let mut enc = tar.into_inner()?;
        enc.try_finish()?;

        Ok(())
    }

    pub fn unpack(&self, archive_path: &Path, dest_dir: &Path, verbose: bool) -> Result<()> {
        if verbose {
            println!(
                "\n{} Extracting package to: {}",
                "📦".bold().blue(),
                dest_dir.display().to_string().bright_black()
            );
            println!("{}", "-".repeat(80));
        }

        let file = File::open(archive_path)
            .with_context(|| format!("No se pudo abrir el archivo tar.gz: {:?}", archive_path))?;

        self.unpack_from_reader(file, dest_dir, verbose)?;

        if verbose {
            println!("{}", "-".repeat(80));
            println!("{} Extraction complete!", "✨".green());
        }

        Ok(())
    }

    pub fn unpack_from_reader<R: Read>(&self, reader: R, dest_dir: &Path, verbose: bool) -> Result<()> {
        let tar = GzDecoder::new(reader);
        let mut archive = Archive::new(tar);

        for entry in archive.entries()? {
            let mut entry = entry.context("Error leyendo entrada del tar")?;

            let path = entry.path()?.into_owned();

            if verbose {
                let size = entry.header().size().unwrap_or(0);
                println!(
                    "{} {:<60} {:>15}",
                    "✓".green(),
                    path.display(),
                    format_bytes(size as usize).bright_black()
                );
            }

            entry.unpack_in(dest_dir).context("Error extrayendo archivo")?;
        }

        Ok(())
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
