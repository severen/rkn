// SPDX-FileCopyrightText: 2025 Severen Redwood <sev@severen.dev>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use clap::Parser;
use mimalloc::MiMalloc;
use rkn::{eval, syntax::parse};
use rustyline::{DefaultEditor, config::Configurer, error::ReadlineError};

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
  if !args.expr.is_empty() {
    run(&args.expr.join(""))?;
  } else {
    repl()?;
  }

  Ok(())
}

fn run(expr: &str) -> Result<()> {
  let (output, errs) = parse(expr).into_output_errors();
  if !errs.is_empty() {
    println!("{:?}", errs);
  }

  if let Some(expr) = output {
    println!("Parse tree: {:?}", expr);
    println!("Result: {}", eval(expr));
  }

  Ok(())
}

fn repl() -> Result<()> {
  let mut rl = DefaultEditor::new()?;
  rl.set_auto_add_history(true);

  loop {
    match rl.readline("> ") {
      Ok(line) => run(&line)?,
      Err(ReadlineError::Eof | ReadlineError::Interrupted) => break,
      Err(err) => {
        println!("Error: {:?}", err);
        break;
      },
    }
  }

  Ok(())
}
