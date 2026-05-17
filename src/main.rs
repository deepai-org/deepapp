use anyhow::Context;
use clap::{Parser, Subcommand};
use deepapp::{compile_file, format_errors, parse, runtime_from_file};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "deep")]
#[command(about = "DeepApp v2 compiler prototype")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Check {
        input: PathBuf,
    },
    Build {
        input: PathBuf,
        #[arg(short, long, default_value = "build")]
        out: PathBuf,
    },
    Migrate {
        input: PathBuf,
        #[arg(long)]
        preview: bool,
    },
    Ast {
        input: PathBuf,
    },
    Run {
        input: PathBuf,
        #[arg(long, default_value = "0.0.0.0:8080")]
        addr: String,
        #[arg(long)]
        snapshot: Option<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Check { input } => {
            let source = fs::read_to_string(&input)
                .with_context(|| format!("failed to read {}", input.display()))?;
            match deepapp::compile_source(&source) {
                Ok(_) => {
                    println!("DeepApp check passed: {}", input.display());
                    Ok(())
                }
                Err(errors) => {
                    eprintln!("{}", format_errors(&errors));
                    std::process::exit(1);
                }
            }
        }
        Command::Build { input, out } => {
            let artifacts = compile_file(&input)?;
            fs::create_dir_all(&out)?;
            write_json(out.join("manifest.json"), &artifacts.manifest)?;
            write_json(out.join("static-report.json"), &artifacts.report)?;
            write_json(out.join("runtime-bundle.json"), &artifacts.runtime)?;
            write_json(out.join("migrations.json"), &artifacts.migrations)?;
            write_json(out.join("sql-plan.json"), &artifacts.sql_plan)?;
            write_json(out.join("storage-catalog.json"), &artifacts.storage_catalog)?;
            write_json(out.join("frontend-assets.json"), &artifacts.frontend_assets)?;
            write_json(out.join("task-catalog.json"), &artifacts.task_catalog)?;
            write_json(out.join("model-catalog.json"), &artifacts.model_catalog)?;
            write_json(out.join("pricing-catalog.json"), &artifacts.pricing_catalog)?;
            println!("DeepApp build wrote {}", out.display());
            Ok(())
        }
        Command::Migrate { input, preview } => {
            let artifacts = compile_file(&input)?;
            if preview {
                println!("{}", serde_json::to_string_pretty(&artifacts.migrations)?);
            } else {
                println!(
                    "Migration apply is intentionally gated; run with --preview in this prototype."
                );
            }
            Ok(())
        }
        Command::Ast { input } => {
            let source = fs::read_to_string(&input)
                .with_context(|| format!("failed to read {}", input.display()))?;
            let program = parse(&source)?;
            println!("{}", serde_json::to_string_pretty(&program)?);
            Ok(())
        }
        Command::Run {
            input,
            addr,
            snapshot,
        } => {
            let runtime = runtime_from_file(&input)?;
            if let Some(snapshot) = snapshot {
                runtime.serve_with_snapshot(&addr, &snapshot)
            } else {
                runtime.serve(&addr)
            }
        }
    }
}

fn write_json(path: PathBuf, value: &impl serde::Serialize) -> anyhow::Result<()> {
    fs::write(&path, serde_json::to_string_pretty(value)?)
        .with_context(|| format!("failed to write {}", path.display()))
}
