// SPDX-FileCopyrightText: 2025 Severen Redwood <sev@severen.dev>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use chumsky::{pratt::*, prelude::*};
use clap::Parser as _;
use mimalloc::MiMalloc;
use rustyline::{DefaultEditor, config::Configurer, error::ReadlineError};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// Advanced scientific calculator for the terminal.
#[derive(clap::Parser)]
#[command(author, version, about)]
struct Args {
  #[arg(
    name = "EXPR",
    help = "An expression to execute",
    trailing_var_arg = true
  )]
  expr: Vec<String>,
}

#[derive(Debug)]
enum Expr {
  // TODO: Use arbitrary precision integers based on `rkn::natural::Natural`.
  Literal(i64),
  Neg(Box<Self>),
  Add(Box<Self>, Box<Self>),
  Sub(Box<Self>, Box<Self>),
  Mul(Box<Self>, Box<Self>),
  Pow(Box<Self>, Box<Self>),
}

fn parser<'src>() -> impl Parser<'src, &'src str, Expr> {
  use Expr::*;

  let number = text::digits(10)
    .to_slice()
    .map(|s: &str| Literal(s.parse::<i64>().unwrap()));

  let op = |c| just(c);

  recursive(|expr| {
    let atom = number.or(expr.delimited_by(just('('), just(')'))).padded();

    atom.pratt((
      infix(left(1), op('+'), |a, _, b, _| Add(Box::new(a), Box::new(b))),
      infix(left(1), op('-'), |a, _, b, _| Sub(Box::new(a), Box::new(b))),
      infix(left(2), op('*'), |a, _, b, _| Mul(Box::new(a), Box::new(b))),
      infix(left(3), op('^'), |a, _, b, _| Pow(Box::new(a), Box::new(b))),
      prefix(2, op('-'), |_, x, _| Neg(Box::new(x))),
    ))
  })
  .then_ignore(end())
}

fn eval(expr: Expr) -> i64 {
  use Expr::*;

  match expr {
    Literal(n) => n,
    Neg(e) => -eval(*e),
    Add(l, r) => eval(*l) + eval(*r),
    Sub(l, r) => eval(*l) - eval(*r),
    Mul(l, r) => eval(*l) * eval(*r),
    // TODO: Support negative exponents.
    Pow(b, e) => {
      eval(*b).pow(eval(*e).try_into().expect("exponents must be positive"))
    },
  }
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
  let (output, errs) = parser().parse(expr).into_output_errors();
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
