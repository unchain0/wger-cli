use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde_json::Value;
use wger_cli::{parse_query, Client};

/// CLI and MCP stdio server for the wger REST API (<https://wger.de/api/v2/>).
///
/// Auth: set `WGER_API_TOKEN` (wger account > API key). Reads of public data work without it.
/// `WGER_BASE_URL` overrides the instance (default <https://wger.de>).
#[derive(Parser)]
#[command(name = "wger", version)]
struct Cli {
    /// Compact JSON output (machine-readable)
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// List a resource, e.g. `wger list workoutlog -q routine=80246 -q ordering=-date`
    List {
        resource: String,
        #[arg(short, long)]
        query: Vec<String>,
        /// follow pagination
        #[arg(long)]
        all: bool,
    },
    /// Get one object by id
    Get { resource: String, id: String },
    /// Create an object (POST) from a JSON body
    Create {
        resource: String,
        #[arg(long)]
        data: String,
    },
    /// Partially update an object (PATCH) with a JSON body
    Update {
        resource: String,
        id: String,
        #[arg(long)]
        data: String,
    },
    /// Delete an object (requires --yes)
    Delete {
        resource: String,
        id: String,
        #[arg(long)]
        yes: bool,
    },
    /// Routine view: structure | logs | stats | date-sequence-display | date-sequence-gym
    Routine {
        id: u64,
        #[arg(default_value = "structure")]
        view: String,
    },
    /// Workout logs (shortcut for `list workoutlog`)
    Logs {
        #[arg(long)]
        routine: Option<u64>,
        #[arg(long)]
        exercise: Option<u64>,
        #[arg(long)]
        since: Option<String>,
        #[arg(long)]
        all: bool,
    },
    /// Create or update an exercise translation (e.g. --lang pt)
    Translation {
        #[arg(long)]
        exercise: u64,
        #[arg(long, default_value = "pt")]
        lang: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        description: Option<String>,
        /// read description (HTML) from file
        #[arg(long)]
        description_file: Option<String>,
        #[arg(long)]
        license_author: Option<String>,
    },
    /// List allowed resources
    Resources,
    /// Run as an MCP server over stdio
    Mcp,
}

fn parse_data(s: &str) -> Result<Value> {
    let raw = if let Some(p) = s.strip_prefix('@') {
        std::fs::read_to_string(p)?
    } else {
        s.to_string()
    };
    serde_json::from_str(&raw).context("--data must be valid JSON (or @file.json)")
}

fn main() {
    let cli = Cli::parse();
    let c = Client::from_env();
    let r = run(&c, cli.cmd);
    match r {
        Ok(Some(v)) => {
            if cli.json {
                println!("{v}");
            } else {
                match serde_json::to_string_pretty(&v) {
                    Ok(s) => println!("{s}"),
                    Err(e) => {
                        eprintln!("error: {e:#}");
                        std::process::exit(1);
                    }
                }
            }
        }
        Ok(None) => {}
        Err(e) => {
            eprintln!("error: {e:#}");
            std::process::exit(1);
        }
    }
}

fn run(c: &Client, cmd: Cmd) -> Result<Option<Value>> {
    Ok(Some(match cmd {
        Cmd::List {
            resource,
            query,
            all,
        } => c.list(&resource, &parse_query(&query)?, all)?,
        Cmd::Get { resource, id } => c.get(&resource, &id)?,
        Cmd::Create { resource, data } => c.create(&resource, &parse_data(&data)?)?,
        Cmd::Update { resource, id, data } => c.update(&resource, &id, &parse_data(&data)?)?,
        Cmd::Delete { resource, id, yes } => c.delete(&resource, &id, yes)?,
        Cmd::Routine { id, view } => c.routine_view(id, &view)?,
        Cmd::Logs {
            routine,
            exercise,
            since,
            all,
        } => {
            let mut q = vec![("ordering".to_string(), "-date".to_string())];
            if let Some(r) = routine {
                q.push(("routine".into(), r.to_string()));
            }
            if let Some(e) = exercise {
                q.push(("exercise".into(), e.to_string()));
            }
            if let Some(s) = since {
                q.push(("date__gte".into(), s));
            }
            c.list("workoutlog", &q, all)?
        }
        Cmd::Translation {
            exercise,
            lang,
            name,
            description,
            description_file,
            license_author,
        } => {
            let desc = match description_file {
                Some(f) => Some(std::fs::read_to_string(f)?),
                None => description,
            };
            c.translation_set(
                exercise,
                &lang,
                name.as_deref(),
                desc.as_deref(),
                license_author.as_deref(),
            )?
        }
        Cmd::Resources => serde_json::json!(wger_cli::RESOURCES),
        Cmd::Mcp => {
            wger_cli::mcp::serve(c)?;
            return Ok(None);
        }
    }))
}
