use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;

mod commands;
mod core;
mod utils;
mod ui;

#[derive(Parser)]
#[command(name = "contexthub")]
#[command(version = "0.1.0")]
#[command(about = "Global Context Storage for AI Coding Assistants", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    Sync {
        #[arg(short, long)]
        path: Option<PathBuf>,
        #[arg(short, long)]
        from: Option<String>,
        #[arg(short, long)]
        last: Option<usize>,
    },
    Context {
        #[arg(short, long)]
        path: Option<PathBuf>,
        #[arg(short, long)]
        export: Option<String>,
    },
    Memory {
        #[arg(short, long)]
        path: Option<PathBuf>,
        #[command(subcommand)]
        subcommand: Option<MemoryCommands>,
    },
    Config {
        #[arg(short, long)]
        path: Option<PathBuf>,
        #[command(subcommand)]
        subcommand: Option<ConfigCommands>,
    },
    Hook {
        #[arg(short, long)]
        path: Option<PathBuf>,
        #[command(subcommand)]
        command: HookCommands,
    },
    Doctor {
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    Status {
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum MemoryCommands {
    Ttl {
        #[arg(long)]
        clear: bool,
        #[arg(long)]
        set_ttl: Option<i32>,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    Show {},
    SetModel {
        model: String,
    },
    SetOllamaUrl {
        url: String,
    },
}

#[derive(Subcommand)]
enum HookCommands {
    Install,
    Uninstall,
}

fn get_repo_path(path: Option<PathBuf>) -> PathBuf {
    path.unwrap_or_else(|| std::env::current_dir().unwrap())
}

fn load_config(path: &PathBuf) -> Result<utils::config::Config> {
    utils::config::Config::load(path)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => {
            let repo_path = get_repo_path(path);
            commands::init::init_repo(&repo_path)?;
        }

        Commands::Sync { path, from, last } => {
            let repo_path = get_repo_path(path);
            let config = load_config(&repo_path)?;
            commands::sync::sync_context(&repo_path, &config, from, last).await?;
        }

        Commands::Context { path, export } => {
            let repo_path = get_repo_path(path);
            let config = load_config(&repo_path)?;
            
            if let Some(format) = export {
                commands::context::export_context(&repo_path, &config, &format)?;
            } else {
                commands::context::display_context(&repo_path, &config)?;
            }
        }

        Commands::Memory { path, subcommand } => {
            let repo_path = get_repo_path(path);
            let mut config = load_config(&repo_path)?;
            
            match subcommand {
                Some(MemoryCommands::Ttl { clear, set_ttl }) => {
                    if clear {
                        commands::memory::clear_ttl_memory(&repo_path, &config)?;
                    } else if let Some(days) = set_ttl {
                        commands::memory::set_ttl(&repo_path, &mut config, days)?;
                    } else {
                        commands::memory::display_ttl_memory(&repo_path, &config)?;
                    }
                }
                None => {
                    commands::memory::display_ttl_memory(&repo_path, &config)?;
                }
            }
        }

        Commands::Config { path, subcommand } => {
            let repo_path = get_repo_path(path);
            let mut config = load_config(&repo_path)?;
            
            match subcommand {
                Some(ConfigCommands::Show { }) => {
                    commands::config_cmd::show_config(&config)?;
                }
                Some(ConfigCommands::SetModel { model }) => {
                    commands::config_cmd::set_config_model(&repo_path, &mut config, model)?;
                }
                Some(ConfigCommands::SetOllamaUrl { url }) => {
                    commands::config_cmd::set_config_ollama_url(&repo_path, &mut config, url)?;
                }
                None => {
                    commands::config_cmd::show_config(&config)?;
                }
            }
        }

        Commands::Hook { path, command } => {
            let repo_path = get_repo_path(path);
            
            match command {
                HookCommands::Install => {
                    commands::hook::install_hook(&repo_path)?;
                }
                HookCommands::Uninstall => {
                    commands::hook::uninstall_hook(&repo_path)?;
                }
            }
        }

        Commands::Doctor { path } => {
            let repo_path = get_repo_path(path);
            let config = load_config(&repo_path)?;
            commands::doctor::doctor(&repo_path, &config)?;
        }

        Commands::Status { path } => {
            let repo_path = get_repo_path(path);
            let config = load_config(&repo_path)?;
            commands::sync::get_sync_status(&repo_path, &config)?;
        }
    }

    Ok(())
}
