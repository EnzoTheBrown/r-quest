mod cli;
mod consts;
mod db;
mod dragon;
mod env;
mod loader;
mod runner;
mod script;
mod share;
mod template;

#[tokio::main]
async fn main() {
    if let Err(e) = cli::handle().await {
        use console::style;
        eprintln!(
            "{}\n{}\n{}",
            style("🔥  Your Qwest angered the dragon!").red().bold(),
            dragon::DRAGON,
            style(format!("{e:?}")).yellow()
        );
        std::process::exit(1);
    }
}
