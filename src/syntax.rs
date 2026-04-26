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

  let number = {
    let hex = just("0x")
      .ignore_then(text::digits(16).to_slice())
      .map(|s| i64::from_str_radix(s, 16).unwrap());

    let octal = just("0o")
      .ignore_then(text::digits(8).to_slice())
      .map(|s| i64::from_str_radix(s, 8).unwrap());

    let binary = just("0b")
      .ignore_then(text::digits(2).to_slice())
      .map(|s| i64::from_str_radix(s, 2).unwrap());

    let decimal = text::digits(10)
      .to_slice()
      .map(|s: &str| s.parse::<i64>().unwrap());

    hex
      .or(octal)
      .or(binary)
      .or(decimal)
      .map(Literal)
  };

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

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::eval;

    fn parse_and_eval(input: &str) -> i64 {
        let (expr, errs) = parse(input).into_output_errors();
        assert!(errs.is_empty());
        eval(expr.unwrap())
    }

    #[test]
    fn test_parse_binary() {
        assert_eq!(parse_and_eval("0b1010"), 10);
        assert_eq!(parse_and_eval("0b1111"), 15);
    }

    #[test]
    fn test_parse_octal() {
        assert_eq!(parse_and_eval("0o12"), 10);
        assert_eq!(parse_and_eval("0o77"), 63);
    }

    #[test]
    fn test_parse_hexadecimal() {
        assert_eq!(parse_and_eval("0x10"), 16);
        assert_eq!(parse_and_eval("0xff"), 255);
        assert_eq!(parse_and_eval("0xCAFE"), 51966);
    }

    #[test]
    fn test_parse_expressions() {
        assert_eq!(parse_and_eval("0b10 + 0o10 + 0x10"), 2 + 8 + 16);
        assert_eq!(parse_and_eval("0b10 * 0o10 - 0x10"), 2 * 8 - 16);
    }

    #[test]
    fn test_invalid_literals() {
        assert!(parse("0b2").has_errors());
        assert!(parse("0o8").has_errors());
        assert!(parse("0xg").has_errors());
        assert!(parse("0b").has_errors());
        assert!(parse("0o").has_errors());
        assert!(parse("0x").has_errors());
    }
}
