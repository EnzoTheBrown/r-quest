use anyhow::{bail, Context, Error, Result};
use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use crate::loader::Header;

const CONFIG_FILES_LOCATION: &str = "/home/enzo/.config/quest";

use crate::{loader, runner};
fn pick<'a>(all: &'a [loader::Request], keys: &[String]) -> Result<Vec<&'a loader::Request>> {
    let mut out = Vec::new();
    for key in keys {
        if let Ok(idx) = key.parse::<usize>() {
            out.push(
                all.get(idx)
                    .with_context(|| format!("No request at index {idx}"))?,
            );
        } else {
            out.push(
                all.iter()
                    .find(|r| r.name == *key)
                    .with_context(|| format!("No request named '{key}'"))?,
            );
        }
    }
    Ok(out)
}

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[arg(short, long, global = true)]
    name: Option<String>,
    #[arg(short = 'e', long = "env-file", global = true)]
    env_file: Option<String>,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    List,
    Describe {
        #[arg(required = true)]
        name: String,
    },
    Run {
        name: String,
        endpoint_name: String,
    },
    Create {
        #[arg(required = true)]
        name: String,
    },
    Edit {
        #[arg(required = true)]
        name: String,
    },
    Delete {
        #[arg(required = true)]
        name: String,
    },
}

fn get_config(name: Option<String>) -> Result<loader::Config, Error> {
    match name {
        Some(name) => {
            let config_path = format!("{CONFIG_FILES_LOCATION}/{}.toml", name);
            crate::loader::load_config(&config_path)
        }
        None => {
            println!("No config file specified, using default.");
            let default_path = format!("{CONFIG_FILES_LOCATION}/default.toml");
            crate::loader::load_config(&default_path)
        }
    }
}

pub async fn handle() -> Result<()> {
    let cli = Cli::parse();
    if let Some(path) = cli.env_file {
        dotenvy::from_path(&path).with_context(|| format!("loading env file {path}"))?;
    } else {
        let _ = dotenvy::dotenv();
    }

    match cli.cmd {
        Cmd::List => {
            let paths = fs::read_dir(CONFIG_FILES_LOCATION).unwrap();

            for path in paths {
                if path.as_ref().unwrap().path().extension() != Some(std::ffi::OsStr::new("toml")) {
                    continue;
                }
                println!("- {}", path.unwrap().path().display())
            }
        }

        Cmd::Describe { name } => {
            let cfg = get_config(cli.name.clone())?;
            println!("Configuration for '{}':", cfg.api.name);
            if let Some(desc) = &cfg.api.description {
                println!("   {}", desc);
            } else {
                println!("No description provided.");
            }
            for r in &cfg.requests {
                println!("┌─ {}", r.name);
                println!("│ method : {}", r.method);
                println!("│ path   : {}", r.path);
                if !r.headers.is_empty() {
                    println!("│ headers:");
                    for Header { key, value } in &r.headers {
                        println!("│   {}: {}", key, value);
                    }
                }
                if let Some(b) = &r.body {
                    println!("│ body   : {}", serde_json::to_string_pretty(b)?);
                }
                println!("└──────────────────────────────\n");
            }
        }

        Cmd::Run {
            name,
            endpoint_name,
        } => {
            let cfg = get_config(cli.name.clone())?;
            let target = cfg.requests.iter().find(|req| req.name == endpoint_name);
            match target {
                Some(req) => {
                    runner::run_single_request(&cfg.api.base_url, &req).await?;
                }
                None => bail!("No request named '{name}' found in the config"),
            }
        }

        Cmd::Create { name } => {
            let mut path = PathBuf::from(CONFIG_FILES_LOCATION);
            path.push(format!("{name}.toml"));
            let mut file = File::create(&path)?;
            let template = r#"[api]
name = "{name}"
base_url = ""

[[request]]
name = "doc"
method = "GET"
path = "/docs""#;
            file.write_all(template.as_bytes())?;

            let status = Command::new("nvim").arg(&path).status()?;

            if !status.success() {
                eprintln!("Editor exited with a non-zero status");
            }
        }
        Cmd::Edit { name } => {
            let mut path = PathBuf::from(CONFIG_FILES_LOCATION);
            path.push(format!("{name}.toml"));
            let status = Command::new("nvim").arg(&path).status()?;

            if !status.success() {
                eprintln!("Editor exited with a non-zero status");
            }
        }
        Cmd::Delete { name } => {
            let mut path = PathBuf::from(CONFIG_FILES_LOCATION);
            path.push(format!("{name}.toml"));
            if path.exists() {
                fs::remove_file(&path)
                    .with_context(|| format!("removing file {}", path.display()))?;
                println!("Deleted config file: {}", path.display());
            } else {
                eprintln!("Config file {} does not exist", path.display());
            }
        }
    }
    Ok(())
}
