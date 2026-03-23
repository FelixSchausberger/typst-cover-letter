//! One-time migration from @preview/appreciated-letter to @local/cover-letter.
//!
//! TEMPORARY — delete this file and its Commands::Migrate arm in cli.rs after migration.

use crate::cli::MigrateArgs;
use anyhow::{Context, Result};
use similar::{ChangeTag, TextDiff};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn run(args: MigrateArgs) -> Result<()> {
    let files: Vec<PathBuf> = WalkDir::new(&args.path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "typ"))
        .map(|e| e.path().to_path_buf())
        .collect();

    if files.is_empty() {
        println!("No .typ files found under {}", args.path.display());
        return Ok(());
    }

    let mut modified = 0usize;
    let mut skipped = 0usize;

    for path in &files {
        let original = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let (migrated, changed) = migrate_content(&original);

        if !changed {
            log::debug!("Skipping {} (no changes needed)", path.display());
            skipped += 1;
            continue;
        }

        if args.dry_run {
            print_diff(path, &original, &migrated);
        } else {
            std::fs::write(path, &migrated)
                .with_context(|| format!("Failed to write {}", path.display()))?;
            println!("Migrated: {}", path.display());
        }
        modified += 1;
    }

    if args.dry_run {
        println!(
            "\n[dry-run] {} files would be modified, {} unchanged",
            modified, skipped
        );
    } else {
        println!("\n{} files migrated, {} unchanged", modified, skipped);
    }

    Ok(())
}

/// Apply all migration transformations to a file's content.
/// Returns (new_content, was_changed).
fn migrate_content(content: &str) -> (String, bool) {
    let mut result = String::with_capacity(content.len());
    let mut changed = false;

    for line in content.lines() {
        // 1. Replace import line
        if line.contains("@preview/appreciated-letter:0.1.0") {
            let new_line = line
                .replace(
                    "@preview/appreciated-letter:0.1.0",
                    "@local/cover-letter:0.1.0",
                )
                // handles `letter` → `cover-letter` in the import list
                .replace(": letter", ": cover-letter");
            result.push_str(&new_line);
            result.push('\n');
            changed = true;
            continue;
        }

        // 2. Replace show rule call
        if line.contains("letter.with(") && !line.contains("cover-letter.with(") {
            let new_line = line.replace("letter.with(", "cover-letter.with(");
            result.push_str(&new_line);
            result.push('\n');
            changed = true;
            continue;
        }

        // 3. Remove name: [Felix Schausberger] line (always redundant with defaults.toml)
        if line.trim().starts_with("name:") && line.contains("Felix Schausberger") {
            changed = true;
            continue; // drop the line
        }

        result.push_str(line);
        result.push('\n');
    }

    (result, changed)
}

fn print_diff(path: &Path, original: &str, migrated: &str) {
    println!("--- {}", path.display());
    let diff = TextDiff::from_lines(original, migrated);
    for change in diff.iter_all_changes() {
        let prefix = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        print!("{}{}", prefix, change);
    }
    println!();
}
