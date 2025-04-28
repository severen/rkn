// SPDX-FileCopyrightText: 2025 Severen Redwood <sev@severen.dev>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{Context, Error, Result, anyhow};
use clap::Parser;
use directories::ProjectDirs;
use mimalloc::MiMalloc;
use rkn::{eval, syntax::parse};
use rustyline::{
  DefaultEditor, config::Configurer, error::ReadlineError, history::History,
};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// Advanced scientific calculator for the terminal.
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
  #[arg(
    name = "EXPR",
    help = "An expression to execute",
    trailing_var_arg = true
  )]
  expr: Vec<String>,
}

fn main() -> Result<()> {
  let args = Args::parse();
  if !args.expr.is_empty() { run(&args.expr.join("")) } else { repl() }
}

fn run(expr: &str) -> Result<()> {
  let (output, errs) = parse(expr).into_output_errors();
  if !errs.is_empty() {
    println!("{errs:?}");
  }

  if let Some(expr) = output {
    println!("Parse tree: {expr:?}");
    println!("Result: {}", eval(expr));
  }

  Ok(())
}

fn repl() -> Result<()> {
  // The second parameter is the 'organisation' name and is left blank because
  // it doesn't really make sense in this context: I'm just one guy writing
  // this!
  let proj_dirs = ProjectDirs::from("dev.severen", "", "rkn")
    .ok_or_else(|| anyhow!("Could not determine home directory"))?;

  // This boils down to using the application state directory per the XDG Base
  // Directory specification on Linux and the application data directory on
  // macOS and Windows since the latter two have no notion of a separate state
  // directory.
  let state_dir = proj_dirs.state_dir().unwrap_or_else(|| proj_dirs.data_dir());
  let history_path = state_dir.join("history.txt");

  let mut rl = DefaultEditor::new()?;
  rl.set_auto_add_history(true);

  match rl.load_history(&history_path) {
    Ok(_) => {},
    Err(ReadlineError::Io(err))
      if err.kind() == std::io::ErrorKind::NotFound =>
    {
      // Deliberately ignore a missing history file: we will create it later if
      // necessary.
    },
    Err(err) => {
      eprintln!(
        "Warning: {:#}",
        Error::new(err).context(format!(
          "Failed to load history from '{}'",
          history_path.display()
        ))
      );
    },
  }

  loop {
    match rl.readline("> ") {
      Ok(line) => run(&line)?,
      Err(ReadlineError::Eof | ReadlineError::Interrupted) => break,
      Err(err) => {
        eprintln!("REPL Error: {err:?}");
        break;
      },
    }
  }

  if rl.history().len() > 0
    && let Err(err) = std::fs::create_dir_all(state_dir)
      .with_context(|| {
        format!("Failed to create state directory at '{}'", state_dir.display())
      })
      .and_then(|_| {
        rl.save_history(&history_path).with_context(|| {
          format!("Failed to save history to '{}'", history_path.display())
        })
      })
  {
    eprintln!("Warning: {err:#}");
  }

  Ok(())
}
