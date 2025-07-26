// mod model;
// mod schema;
// mod server;
//
// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     server::run_server().await
// }
// use anyhow::Result;
//
// mod loader;
//
// fn main() -> Result<()> {
//     let cfg = loader::load_config("test_config.toml")?;
//
//     for req in &cfg.requests {
//         println!("─ {} {}", req.method, req.path);
//
//         if let Some(body) = &req.body {
//             if let Some(full_name) = body.get("full_name") {
//                 println!("  ↳ JSON body field 'full_name' = {}", full_name);
//             }
//         }
//     }
//
//     Ok(())
// }

use anyhow::Result;

mod cli;
mod loader;
mod runner;

#[tokio::main]
async fn main() -> Result<()> {
    cli::handle().await
}
