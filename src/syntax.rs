// SPDX-FileCopyrightText: 2025 Severen Redwood <sev@severen.dev>
// SPDX-License-Identifier: GPL-3.0-or-later

use chumsky::{pratt::*, prelude::*};

#[derive(Debug)]
pub enum Expr {
  // TODO: Use arbitrary precision integers based on `rkn::natural::Natural`.
  Literal(i64),
  Neg(Box<Self>),
  Add(Box<Self>, Box<Self>),
  Sub(Box<Self>, Box<Self>),
  Mul(Box<Self>, Box<Self>),
  Pow(Box<Self>, Box<Self>),
}

pub fn parse(input: &str) -> ParseResult<Expr, EmptyErr> {
  parser().parse(input)
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
      infix(right(3), op('^'), |a, _, b, _| Pow(Box::new(a), Box::new(b))),
      prefix(2, op('-'), |_, x, _| Neg(Box::new(x))),
    ))
  })
  .then_ignore(end())
}
