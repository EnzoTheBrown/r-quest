use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, MultiSelect};

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
    #[arg(short, long, global = true, default_value = "test_config.toml")]
    config: String,
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
        names: Vec<String>,
    },
    Run {
        names: Vec<String>,
    },
    Select,
}

pub async fn handle() -> Result<()> {
    let cli = Cli::parse();
    if let Some(path) = cli.env_file {
        dotenvy::from_path(&path).with_context(|| format!("loading env file {path}"))?;
    } else {
        let _ = dotenvy::dotenv();
    }

    let cfg = crate::loader::load_config(&cli.config)?;
    match cli.cmd {
        Cmd::List => {
            for (i, r) in cfg.requests.iter().enumerate() {
                println!("{:>2}: {:<6} {}", i, r.method, r.name);
            }
        }

        Cmd::Describe { names } => {
            for r in pick(&cfg.requests, &names)? {
                println!("┌─ {}", r.name);
                println!("│ method : {}", r.method);
                println!("│ path   : {}", r.path);
                if let Some(h) = &r.headers {
                    println!("│ headers: {h:?}");
                }
                if let Some(b) = &r.body {
                    println!("│ body   : {}", serde_json::to_string_pretty(b)?);
                }
                println!("└──────────────────────────────\n");
            }
        }

        Cmd::Run { names } => {
            let targets = if names.is_empty() {
                cfg.requests.iter().collect()
            } else {
                pick(&cfg.requests, &names)?
            };
            runner::run_requests(&cfg.api.base_url, &targets).await?;
        }

        Cmd::Select => {
            let items: Vec<String> = cfg
                .requests
                .iter()
                .map(|r| format!("{:<6} {}", r.method, r.name))
                .collect();

            let chosen = MultiSelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Select queries to execute")
                .items(&items)
                .interact()?;

            if chosen.is_empty() {
                bail!("Nothing selected – aborting.");
            }
            let targets: Vec<&loader::Request> =
                chosen.into_iter().map(|i| &cfg.requests[i]).collect();
            runner::run_requests(&cfg.api.base_url, &targets).await?;
        }
    }
    Ok(())
}
