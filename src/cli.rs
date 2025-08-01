use crate::{consts::CONFIG_FILES_LOCATION, loader, runner};
use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use console::{style, Emoji};
use indicatif::{ProgressBar, ProgressStyle};
use std::{fs, fs::File, io::Write, path::PathBuf, process::Command};
use tabled::settings::Style as TableStyle;
use tabled::{Table, Tabled};

#[derive(Parser)]
#[command(
    author,
    version,
    about = "🧙‍♂️  A wizardly HTTP spell‑book.  Invoke your Qwest!",
    rename_all = "kebab_case",
    color = clap::ColorChoice::Always
)]
pub struct Cli {
    #[arg(short, long, global = true)]
    book: Option<String>,
    #[arg(short = 'e', long = "env-file", global = true)]
    env_file: Option<String>,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    List,
    Describe {
        name: String,
    },
    Run {
        #[arg(help = "Name of the spell‑book TOML (without .toml)")]
        name: String,
        #[arg(help = "Name of the spell (request) inside the book")]
        spell_name: String,
    },
    Create {
        name: String,
    },
    Edit {
        name: String,
    },
    Delete {
        name: String,
    },
}

fn header<S: AsRef<str>>(emoji: Emoji<'_, '_>, text: S) {
    println!("{} {}", emoji, style(text.as_ref()).bold().cyan());
}

fn list_tomes() -> Result<()> {
    #[derive(Tabled)]
    struct Row {
        #[tabled(rename = "#")]
        idx: usize,
        #[tabled(rename = "Spell‑Book")]
        file: String,
    }

    let files: Vec<_> = fs::read_dir(CONFIG_FILES_LOCATION)
        .with_context(|| "reading config directory")?
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().and_then(|e| e.to_str()) == Some("toml"))
        .collect();

    if files.is_empty() {
        println!(
            "{}",
            style("No spell‑books found – conjure one with `qwest scribe <name>`. ").yellow()
        );
        return Ok(());
    }

    let rows: Vec<Row> = files
        .iter()
        .enumerate()
        .map(|(i, e)| Row {
            idx: i + 1,
            file: e.file_name().to_string_lossy().into(),
        })
        .collect();

    let table = Table::new(rows).with(TableStyle::rounded()).to_string();
    header(Emoji("📜", "[tomes]"), "Available spell‑books");
    println!("{}", table);
    Ok(())
}

fn print_tome(cfg: &loader::Config) -> Result<()> {
    header(
        Emoji("📖", "[tome]"),
        format!("Spell‑book for '{}'", cfg.api.name),
    );

    if let Some(d) = &cfg.api.description {
        println!("  {}", style(d).italic());
    }

    #[derive(Tabled)]
    struct SpellRow {
        #[tabled(rename = "#")]
        idx: usize,
        name: String,
        method: String,
        path: String,
    }

    let rows: Vec<SpellRow> = cfg
        .requests
        .iter()
        .enumerate()
        .map(|(i, r)| SpellRow {
            idx: i + 1,
            name: r.name.clone(),
            method: r.method.to_string(),
            path: r.path.clone(),
        })
        .collect();

    let table = Table::new(rows).with(TableStyle::markdown()).to_string();
    println!("{}", table);
    Ok(())
}

async fn cast_spell(cfg: &loader::Config, spell: &str) -> Result<()> {
    let req = cfg
        .requests
        .iter()
        .find(|r| r.name == spell)
        .with_context(|| format!("No spell named '{spell}'"))?;

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_chars("🪄✨🔮🧙‍♀️"),
    );
    pb.set_message(format!("Conjuring '{}'…", req.name));
    pb.enable_steady_tick(std::time::Duration::from_millis(120));

    runner::run_single_request(&cfg.api.base_url, req).await?;
    pb.finish_with_message("Spell resolved ✅");
    Ok(())
}

fn load_tome(name: Option<String>) -> Result<loader::Config> {
    let fname = name.unwrap_or_else(|| "default".into());
    let path = format!("{}/{}.toml", CONFIG_FILES_LOCATION, fname);
    loader::load_config(&path).with_context(|| format!("loading spell‑book '{}.toml'", fname))
}

pub async fn handle() -> Result<()> {
    let cli = Cli::parse();

    if let Some(path) = cli.env_file {
        dotenvy::from_path(&path).with_context(|| format!("loading env file {path}"))?;
    } else {
        let _ = dotenvy::dotenv();
    }

    match cli.cmd {
        Cmd::List => list_tomes()?,
        Cmd::Describe { name } => {
            let cfg = load_tome(cli.book)?;
            if cfg.api.name != name {
                println!(
                    "{}",
                    style("Warning: requested name does not match book's api.name").yellow()
                );
            }
            print_tome(&cfg)?;
        }
        Cmd::Run { name, spell_name } => {
            let cfg = load_tome(Some(name))?;
            cast_spell(&cfg, &spell_name).await?;
        }
        Cmd::Create { name } => {
            let mut path = PathBuf::from(CONFIG_FILES_LOCATION);
            path.push(format!("{name}.toml"));
            if path.exists() {
                bail!("Spell‑book {} already exists", path.display());
            }

            let mut file = File::create(&path)?;
            let template = format!(
                r#"# By decree of the Arcane Council this spell‑book belongs to $(USER)
[api]
name = "{name}"
base_url = ""

[[requests]]
name   = "docs"
method = "GET"
path   = "/docs"
"#,
                name = name
            );
            file.write_all(template.as_bytes())?;

            header(
                Emoji("✨", "[scribe]"),
                "Spell‑book conjured – opening $EDITOR",
            );
            let status = Command::new(std::env::var("EDITOR").unwrap_or_else(|_| "nvim".into()))
                .arg(&path)
                .status()?;
            if !status.success() {
                eprintln!(
                    "{}",
                    style("Editor exited with non‑zero status – tome is saved though.").red()
                );
            }
        }
        Cmd::Edit { name } => {
            let mut path = PathBuf::from(CONFIG_FILES_LOCATION);
            path.push(format!("{name}.toml"));
            let status = Command::new(std::env::var("EDITOR").unwrap_or_else(|_| "nvim".into()))
                .arg(&path)
                .status()?;
            if !status.success() {
                eprintln!("{}", style("Editor exited with non‑zero status").red());
            }
        }
        Cmd::Delete { name } => {
            let mut path = PathBuf::from(CONFIG_FILES_LOCATION);
            path.push(format!("{name}.toml"));
            if path.exists() {
                fs::remove_file(&path)
                    .with_context(|| format!("banishing file {}", path.display()))?;
                println!("{}", style(format!("Banished {}", path.display())).red());
            } else {
                eprintln!("{} does not exist", path.display());
            }
        }
    }
    Ok(())
}
