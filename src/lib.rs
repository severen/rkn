// SPDX-FileCopyrightText: 2025 Severen Redwood <sev@severen.dev>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(bigint_helper_methods)]

use crate::syntax::Expr;

pub mod natural;
pub mod syntax;

pub fn eval(expr: Expr) -> i64 {
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
