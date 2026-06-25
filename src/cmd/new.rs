use crate::cli::NewArgs;
use crate::template::{self, TemplateArgs};
use anyhow::{Context, Result};
use chrono::Local;
use dialoguer::{Confirm, Input, Select};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(args: NewArgs) -> Result<()> {
    let company = match &args.company {
        Some(c) => c.clone(),
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
    let (contact, contact_gender) = if contact.is_empty() {
        (None, None)
    } else {
        let genders = &["Female", "Male"];
        let idx = Select::new()
            .with_prompt("Gender")
            .items(genders)
            .default(0)
            .interact()?;
        (Some(contact), Some(["female", "male"][idx]))
    };

    let position = match &args.position {
        Some(p) => p.clone(),
        None => Input::new()
            .with_prompt("Job position / title")
            .interact_text()?,
    };

    let lang = match &args.lang {
        Some(l) => l.clone(),
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

    let sender_name = match &args.sender_name {
        Some(n) => n.clone(),
        None => Input::new().with_prompt("Your name").interact_text()?,
    };

    let (sender_street, sender_city) = resolve_sender_address(&args)?;

    let today = Local::now().format("%d.%m.%Y").to_string();
    let date = match &args.date {
        Some(d) => d.clone(),
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
    let base = args.dir.unwrap_or_else(|| {
        crate::config::load()
            .ok()
            .and_then(|c| c.output?.dir.map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("."))
    });
    let app_dir = base.join(&dir_name);
    let typ_file = app_dir.join("Cover_letter_Schausberger.typ");

    std::fs::create_dir_all(&app_dir)
        .with_context(|| format!("Failed to create directory {}", app_dir.display()))?;

    let content = template::render(&TemplateArgs {
        company: &company,
        address: &address,
        contact: contact.as_deref(),
        contact_gender: contact_gender,
        position: &position,
        lang: &lang,
        date: &date,
        sender_name: &sender_name,
        sender_street: &sender_street,
        sender_city: &sender_city,
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
    }

    Ok(())
}

fn resolve_sender_address(args: &NewArgs) -> Result<(String, String)> {
    // CLI flags take priority
    if let (Some(street), Some(city)) = (&args.sender_street, &args.sender_city) {
        return Ok((street.clone(), city.clone()));
    }

    // Try sops secret next
    if let Ok(addr) = decrypt_sops_address() {
        return Ok(split_address(&addr));
    }

    // Fall back to interactive prompts
    let street = match &args.sender_street {
        Some(s) => s.clone(),
        None => Input::new().with_prompt("Your street").interact_text()?,
    };
    let city = match &args.sender_city {
        Some(c) => c.clone(),
        None => Input::new().with_prompt("Your city").interact_text()?,
    };
    Ok((street, city))
}

fn split_address(addr: &str) -> (String, String) {
    if let Some(idx) = addr.find('\n') {
        let street = addr[..idx].trim().to_string();
        let city = addr[idx + 1..].trim().to_string();
        return (street, city);
    }
    if let Some(idx) = addr.find(", ") {
        let street = addr[..idx].trim().to_string();
        let city = addr[idx + 2..].trim().to_string();
        return (street, city);
    }
    (addr.to_string(), String::new())
}

#[derive(Deserialize)]
struct Secret {
    private: Private,
}

#[derive(Deserialize)]
struct Private {
    address: String,
}

fn decrypt_sops_address() -> Result<String> {
    let secret_path = PathBuf::from("secrets/secrets.yaml");
    if !secret_path.is_file() {
        anyhow::bail!("secrets/secrets.yaml not found");
    }

    let output = Command::new("sops")
        .args(["-d", "secrets/secrets.yaml"])
        .output()
        .context("Failed to run sops — is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("sops decryption failed: {}", stderr.trim());
    }

    let secret: Secret =
        serde_yaml::from_slice(&output.stdout).context("Failed to parse sops output")?;

    Ok(secret.private.address)
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
