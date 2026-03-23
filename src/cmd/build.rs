use crate::cli::BuildArgs;
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use walkdir::WalkDir;

pub fn run(args: BuildArgs) -> Result<()> {
    if args.all {
        let base = args.path.unwrap_or_else(|| PathBuf::from("."));
        build_all(&base, args.force)
    } else {
        let path = args.path.unwrap_or_else(|| PathBuf::from("."));
        let typ_file = resolve_typ_path(&path)?;
        compile_file(&typ_file, args.force)?;
        println!("✓ {}", typ_file.display());
        Ok(())
    }
}

fn build_all(base: &Path, force: bool) -> Result<()> {
    let files: Vec<PathBuf> = WalkDir::new(base)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "typ"))
        .map(|e| e.path().to_path_buf())
        .collect();

    if files.is_empty() {
        println!("No .typ files found under {}", base.display());
        return Ok(());
    }

    // Tag each file as up-to-date before running in parallel
    let tagged: Vec<(PathBuf, bool)> = files
        .iter()
        .map(|f| (f.clone(), !force && is_pdf_up_to_date(f)))
        .collect();

    let results: Vec<(PathBuf, bool, Result<()>)> = tagged
        .par_iter()
        .map(|(f, up_to_date)| (f.clone(), *up_to_date, compile_file(f, force)))
        .collect();

    let mut ok = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;

    for (path, up_to_date, result) in &results {
        match result {
            Ok(()) if *up_to_date => {
                log::debug!("- {} (up to date)", path.display());
                skipped += 1;
            }
            Ok(()) => {
                println!("✓ {}", path.display());
                ok += 1;
            }
            Err(e) => {
                eprintln!("✗ {}: {}", path.display(), e);
                failed += 1;
            }
        }
    }

    println!("\n{} compiled, {} skipped, {} failed", ok, skipped, failed);

    if failed > 0 {
        anyhow::bail!("{} compilation(s) failed", failed);
    }
    Ok(())
}

pub fn compile_file(typ_path: &Path, force: bool) -> Result<()> {
    if !force && is_pdf_up_to_date(typ_path) {
        log::debug!("Skipping {} (PDF is up to date)", typ_path.display());
        return Ok(());
    }

    let output = Command::new("typst")
        .arg("compile")
        .arg(typ_path)
        .output()
        .with_context(|| format!("Failed to run typst on {}", typ_path.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "typst compile failed for {}:\n{}",
            typ_path.display(),
            stderr.trim()
        );
    }

    Ok(())
}

fn is_pdf_up_to_date(typ_path: &Path) -> bool {
    let pdf_path = typ_path.with_extension("pdf");
    let mtime = |p: &Path| -> Option<SystemTime> { p.metadata().ok()?.modified().ok() };
    match (mtime(typ_path), mtime(&pdf_path)) {
        (Some(typ_t), Some(pdf_t)) => pdf_t >= typ_t,
        _ => false,
    }
}

fn resolve_typ_path(path: &Path) -> Result<PathBuf> {
    if path.is_file() {
        return Ok(path.to_path_buf());
    }
    if path.is_dir() {
        let mut candidates: Vec<PathBuf> = std::fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "typ"))
            .collect();

        // Prefer files starting with "Cover_letter_"
        if let Some(pos) = candidates.iter().position(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("Cover_letter_"))
        }) {
            return Ok(candidates.remove(pos));
        }

        match candidates.len() {
            0 => anyhow::bail!("No .typ file found in {}", path.display()),
            _ => return Ok(candidates.remove(0)),
        }
    }
    anyhow::bail!("Path does not exist: {}", path.display())
}
