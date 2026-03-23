use crate::cli::NewArgs;
use crate::template::{self, TemplateArgs};
use anyhow::{Context, Result};
use chrono::Local;
use dialoguer::{Confirm, Input, Select};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(args: NewArgs) -> Result<()> {
    let company = match args.company {
        Some(c) => c,
        None => Input::new().with_prompt("Company").interact_text()?,
    };

    let address: String = Input::new()
        .with_prompt("Recipient address (street \\ city, leave blank if company only)")
        .allow_empty(true)
        .interact_text()?;

    let contact: String = Input::new()
        .with_prompt("Contact person (optional, press Enter to skip)")
        .allow_empty(true)
        .interact_text()?;
    let contact = if contact.is_empty() {
        None
    } else {
        Some(contact)
    };

    let position = match args.position {
        Some(p) => p,
        None => Input::new()
            .with_prompt("Job position / title")
            .interact_text()?,
    };

    let lang = match args.lang {
        Some(l) => l,
        None => {
            let langs = &["de", "en"];
            let idx = Select::new()
                .with_prompt("Language")
                .items(langs)
                .default(0)
                .interact()?;
            langs[idx].to_string()
        }
    };

    let today = Local::now().format("%d.%m.%Y").to_string();
    let date = match args.date {
        Some(d) => d,
        None => Input::new()
            .with_prompt("Date")
            .default(today)
            .interact_text()?,
    };

    // Build output directory name: YYYY_MM_DD_Company_Position
    let dir_date = Local::now().format("%Y_%m_%d").to_string();
    let dir_name = format!(
        "{}_{}_{}",
        dir_date,
        sanitize(&company),
        sanitize(&position)
    );
    let base = args.dir.unwrap_or_else(|| PathBuf::from("."));
    let app_dir = base.join(&dir_name);
    let typ_file = app_dir.join("Cover_letter_Schausberger.typ");

    std::fs::create_dir_all(&app_dir)
        .with_context(|| format!("Failed to create directory {}", app_dir.display()))?;

    let content = template::render(&TemplateArgs {
        company: &company,
        address: &address,
        contact: contact.as_deref(),
        position: &position,
        lang: &lang,
        date: &date,
    });

    std::fs::write(&typ_file, &content)
        .with_context(|| format!("Failed to write {}", typ_file.display()))?;

    println!("Created: {}", typ_file.display());

    open_editor(&typ_file)?;

    let compile = Confirm::new()
        .with_prompt("Compile to PDF now?")
        .default(true)
        .interact()?;

    if compile {
        crate::cmd::build::compile_file(&typ_file, true)?;
        println!(
            "PDF written to {}",
            typ_file.with_extension("pdf").display()
        );
    }

    Ok(())
}

fn open_editor(path: &Path) -> Result<()> {
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "hx".to_string());

    let status = Command::new(&editor)
        .arg(path)
        .status()
        .with_context(|| format!("Failed to launch editor '{}'", editor))?;

    if !status.success() {
        log::warn!("Editor exited with non-zero status: {}", status);
    }
    Ok(())
}

fn sanitize(s: &str) -> String {
    let s = s
        .replace('ä', "ae")
        .replace('Ä', "Ae")
        .replace('ö', "oe")
        .replace('Ö', "Oe")
        .replace('ü', "ue")
        .replace('Ü', "Ue")
        .replace('ß', "ss");
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}
