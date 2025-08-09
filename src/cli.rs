use crate::db;
use crate::env::load_env;
use crate::share::share;
use crate::template::TEMPLATE;
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
    about = "🧙‍♂️  A wizardly HTTP spell‑book. Invoke your Qwest!",
    rename_all = "kebab_case",
    color = clap::ColorChoice::Always
)]
pub struct Cli {
    #[arg(short, long, global = true)]
    book: Option<String>,

    #[arg(short = 'e', long = "env-file", global = true)]
    env_file: Option<String>,

    #[arg(long = "env", global = true, default_value = "default")]
    env_name: String,

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
    Share {
        name: String,
    },
    Vars {
        #[command(subcommand)]
        action: VarsCmd,
    },
}

#[derive(Subcommand)]
enum VarsCmd {
    List {
        #[arg(long = "project")]
        project: String,
    },

    Set {
        #[arg(long = "project")]
        project: String,
        name: String,
        value: String,
    },

    Unset {
        #[arg(long = "project")]
        project: String,
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

async fn cast_spell(
    cfg: &loader::Config,
    spell: &str,
    project_name: &str,
    env_name: &str,
) -> Result<()> {
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

    runner::run_single_request(&cfg.api.base_url, project_name, env_name, req).await?;
    pb.finish_with_message("Spell resolved ✅");
    Ok(())
}

fn load_tome(project_name: Option<String>, env_name: &str) -> Result<loader::Config> {
    let fname = project_name.clone().unwrap_or_else(|| "default".into());
    let path = format!("{}/{}.toml", CONFIG_FILES_LOCATION, fname);
    let mut vars = load_env()?;
    let db_vars = crate::db::load_vars(&fname, env_name).unwrap_or_default();
    for (k, v) in db_vars {
        vars.entry(k).or_insert(v);
    }

    loader::load_config(&path, vars).with_context(|| format!("loading spell-book '{}.toml'", fname))
}

pub async fn handle() -> Result<()> {
    let cli = Cli::parse();
    let env_name = cli.env_name.clone();

    if let Some(path) = cli.env_file {
        dotenvy::from_path(&path).with_context(|| format!("loading env file {path}"))?;
    } else {
        let _ = dotenvy::dotenv();
    }

    match cli.cmd {
        Cmd::List => list_tomes()?,
        Cmd::Describe { name } => {
            let cfg = load_tome(cli.book, &env_name)?;
            if cfg.api.name != name {
                println!(
                    "{}",
                    style("Warning: requested name does not match book's api.name").yellow()
                );
            }
            print_tome(&cfg)?;
        }
        Cmd::Run { name, spell_name } => {
            let cfg = load_tome(Some(name.clone()), &env_name)?;
            cast_spell(&cfg, &spell_name, &name, &env_name).await?;
        }
        Cmd::Create { name } => {
            let mut path = PathBuf::from(CONFIG_FILES_LOCATION);
            path.push(format!("{name}.toml"));
            if path.exists() {
                bail!("Spell‑book {} already exists", path.display());
            }

            let mut file = File::create(&path)?;
            let template = format!(
                r#"
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
        Cmd::Share { name } => {
            println!(
                "http://localhost:8080/config/{}",
                share("http://localhost:8080/config", &name, &TEMPLATE)
                    .await
                    .with_context(|| format!("sharing spell‑book '{}'", name))?
            );
        }
        Cmd::Vars { action } => match action {
            VarsCmd::List { project } => {
                let vars = db::load_vars(&project, &env_name)?;
                if vars.is_empty() {
                    println!(
                        "{}",
                        style(format!(
                            "No variables for project='{project}', env='{env_name}'"
                        ))
                        .yellow()
                    );
                } else {
                    #[derive(Tabled)]
                    struct Row {
                        name: String,
                        value: String,
                    }
                    let rows: Vec<Row> = vars
                        .into_iter()
                        .map(|(k, v)| Row { name: k, value: v })
                        .collect();
                    let table = Table::new(rows).with(TableStyle::rounded()).to_string();
                    header(
                        Emoji("🔑", "[vars]"),
                        format!("Variables for {project} @ {env_name}"),
                    );
                    println!("{table}");
                }
            }
            VarsCmd::Set {
                project,
                name,
                value,
            } => {
                db::upsert_var(&project, &env_name, &name, &value)?;
                println!(
                    "{}",
                    style(format!("Set {name} for {project} @ {env_name}")).green()
                );
            }
            VarsCmd::Unset { project, name } => {
                db::delete_var(&project, &env_name, &name)?;
                println!(
                    "{}",
                    style(format!("Unset {name} for {project} @ {env_name}")).yellow()
                );
            }
        },
    }
    Ok(())
}
