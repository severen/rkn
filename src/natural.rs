// SPDX-FileCopyrightText: 2025 Severen Redwood <sev@severen.dev>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ops::{Add, AddAssign, Mul, MulAssign};

/// A single digit of an arbitrary-precision integer.
///
/// For efficiency reasons, this type is chosen so that each limb is a single
/// machine word on the target architecture, which at the moment is only x86_64.
type Limb = u64;

/// An arbitrary-precision nonnegative integer.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Natural(Repr);

/// The internal representation of a [`Natural`].
///
/// If the number fits within a single [`Limb`], it is stored inline as a single
/// limb and we refer to it as a _small_ natural number. If it doesn't, it is
/// stored as a [`Vec`] of limbs and we refer to it as a _large_ natural number.
///
/// This arrangement allows avoiding allocations in many cases, but with the
/// trade off of branching in each operation according to the representation of
/// the operands.
#[derive(Debug, Clone, Eq, PartialEq)]
enum Repr {
  /// A natural number which fits within a single limb.
  Small(Limb),
  /// A natural number which fits in two or more limbs.
  ///
  /// The limbs are stored in _little-endian_ order, that is, with the limbs
  /// ordered from least-significant to most-significant. Or otherwise put, in
  /// opposite order to how one would conventionally write the digits of a
  /// integer in base 10 (or base 2, 8, 16, and so on).
  ///
  /// More precisely, a large natural consisting of the limbs
  /// `[a₀, a₁, …, aₙ]`, represents the value
  ///
  /// `aₙ * 2^(nw) + ⋯ + a₂ * 2^(2w) + a₁ * 2^w + a₀`
  ///
  /// where `w` is the size of a machine word (typically 64 on today's 64-bit
  /// architectures).
  ///
  /// Note that the backing vector can be assumed to contain two or more limbs
  /// since the natural should be stored in the `Small` variant otherwise.
  Large(Vec<Limb>),
}

impl Natural {
  /// The natural number 0.
  pub const ZERO: Self = Self(Repr::Small(0));
  /// The natural number 1.
  pub const ONE: Self = Self(Repr::Small(1));

  #[cfg(test)]
  fn from_limbs(limbs: &[Limb]) -> Self {
    if limbs.is_empty() {
      Self(Repr::Small(0))
    } else if limbs.len() == 1 {
      Self(Repr::Small(limbs[0]))
    } else {
      Self(Repr::Large(limbs.to_vec()))
    }
  }
}

impl From<Limb> for Natural {
  fn from(value: Limb) -> Self {
    Self(Repr::Small(value))
  }
}

impl Add for Natural {
  type Output = Self;

  fn add(mut self, other: Self) -> Self::Output {
    self += other;
    self
  }
}

impl AddAssign for Natural {
  #[inline]
  fn add_assign(&mut self, mut other: Self) {
    match (&mut self.0, &mut other.0) {
      (Repr::Small(x), Repr::Small(y)) => {
        let (sum, overflow) = x.overflowing_add(*y);
        if overflow {
          *self = Natural(Repr::Large(vec![sum, 1]));
        } else {
          *x = sum;
        }
      },
      (Repr::Small(_), Repr::Large(_)) => {
        // We have ownership of _both_ `self` and `rhs`, so this reduces to the
        // case of adding a large natural to a small one after we swap the two.
        std::mem::swap(self, &mut other);
        *self += other;
      },
      (Repr::Large(x), Repr::Small(y)) => {
        let (sum, mut carry) = x[0].overflowing_add(*y);
        x[0] = sum;

        for limb in x.iter_mut().skip(1) {
          if !carry {
            break;
          }

          let (sum, overflow) = limb.overflowing_add(1);
          *limb = sum;
          carry = overflow;
        }

        if carry {
          x.push(1);
        }
      },
      (Repr::Large(x), Repr::Large(y)) => {
        // We have ownership of _both_ `x` and `y`, so we avoid unnecessarily
        // allocating extra space by ensuring `x` becomes the larger of the two.
        if x.len() < y.len() {
          std::mem::swap(x, y);
        }

        let mut carry = false;

        for (x_limb, y_limb) in x.iter_mut().zip(&mut *y) {
          let (sum, overflow) = x_limb.carrying_add(*y_limb, carry);
          *x_limb = sum;
          carry = overflow;
        }

        // Propagate the carry through the rest of the limbs in `x` if
        // necessary.
        for limb in x.iter_mut().skip(y.len()) {
          if !carry {
            break;
          }

          let (sum, overflow) = limb.overflowing_add(1);
          *limb = sum;
          carry = overflow;
        }

        if carry {
          x.push(1);
        }
      },
    }
  }
}

impl Mul<Natural> for Natural {
  type Output = Self;

  fn mul(mut self, other: Natural) -> Self::Output {
    self *= other;
    self
  }
}

impl MulAssign<Natural> for Natural {
  fn mul_assign(&mut self, mut other: Natural) {
    match (&mut self.0, &mut other.0) {
      (Repr::Small(0), _) => {},
      (_, Repr::Small(0)) => *self = Natural::ZERO,
      (Repr::Small(1), _) => *self = other,
      (_, Repr::Small(1)) => {},
      (Repr::Small(x), Repr::Small(y)) => {
        let (product, overflow) = x.widening_mul(*y);
        if overflow != 0 {
          *self = Natural(Repr::Large(vec![product, overflow]));
        } else {
          *x = product;
        }
      },
      (Repr::Small(_), Repr::Large(_)) => {
        // We have ownership of _both_ `self` and `rhs`, so this reduces to the
        // case of multiplying a large natural by a small one after we swap the
        // two.
        std::mem::swap(self, &mut other);
        *self *= other;
      },
      (Repr::Large(_), Repr::Small(_)) => {
        todo!("Implement multiplication of large natural by small natural")
      },
      (Repr::Large(_), Repr::Large(_)) => {
        todo!("Implement multiplication of large natural by large natural")
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const SMALL_MAX: Natural = Natural(Repr::Small(Limb::MAX));

  /// Assert that a list of equalities on [`Natural`] numbers hold.
  macro_rules! assert_exprs {
    ($($left:literal $op:tt $right:literal = $result:literal),* $(,)?) => {
      $(
        assert_eq!(
          Natural::from($left) $op Natural::from($right),
          Natural::from($result),
        );
      )*
    };
  }

  #[test]
  fn test_add_small_small() {
    assert_exprs! {
      0 + 0 = 0,
      0 + 1 = 1,
      1 + 0 = 1,
      1 + 1 = 2,
      123 + 456 = 579
    };
  }

  #[test]
  fn test_add_small_small_overflow() {
    assert_eq!(SMALL_MAX + Natural::ONE, Natural::from_limbs(&[0, 1]));
    assert_eq!(SMALL_MAX + SMALL_MAX, Natural::from_limbs(&[Limb::MAX - 1, 1]));
  }

  #[test]
  fn test_add_small_large() {
    let small = Natural::from(123);
    let large = Natural::from_limbs(&[456, 1]);
    assert_eq!(small + large, Natural::from_limbs(&[579, 1]));

    let small = Natural::from(42);
    let large = Natural::from_limbs(&[100, 200, 300]);
    assert_eq!(small + large, Natural::from_limbs(&[142, 200, 300]));
  }

  #[test]
  fn test_add_small_large_overflow() {
    let small = Natural::from(1);
    let large = Natural::from_limbs(&[Limb::MAX, 0]);
    assert_eq!(small + large, Natural::from_limbs(&[0, 1]));

    let small = Natural::from(5);
    let large = Natural::from_limbs(&[Limb::MAX - 3, Limb::MAX]);
    assert_eq!(small + large, Natural::from_limbs(&[1, 0, 1]));
  }

  #[test]
  fn test_add_large_small() {
    let large = Natural::from_limbs(&[100, 200]);
    let small = Natural::from(50);
    assert_eq!(large + small, Natural::from_limbs(&[150, 200]));

    let large = Natural::from_limbs(&[1000, 2000, 3000]);
    let small = Natural::from(234);
    assert_eq!(large + small, Natural::from_limbs(&[1234, 2000, 3000]));
  }

  #[test]
  fn test_add_large_small_overflow() {
    let large = Natural::from_limbs(&[Limb::MAX, 10]);
    let small = Natural::from(1);
    assert_eq!(large + small, Natural::from_limbs(&[0, 11]));

    let large = Natural::from_limbs(&[Limb::MAX, Limb::MAX]);
    let small = Natural::from(1);
    assert_eq!(large + small, Natural::from_limbs(&[0, 0, 1]));
  }

  #[test]
  fn test_add_large_large() {
    let a = Natural::from_limbs(&[123, 456]);
    let b = Natural::from_limbs(&[789, 123]);
    assert_eq!(a + b, Natural::from_limbs(&[912, 579]));

    let a = Natural::from_limbs(&[100, 200, 300]);
    let b = Natural::from_limbs(&[400, 500]);
    assert_eq!(a + b, Natural::from_limbs(&[500, 700, 300]));
  }

  #[test]
  fn test_add_large_large_overflow() {
    let a = Natural::from_limbs(&[Limb::MAX, 10]);
    let b = Natural::from_limbs(&[1, 5]);
    assert_eq!(a + b, Natural::from_limbs(&[0, 16]));

    let a = Natural::from_limbs(&[Limb::MAX, Limb::MAX]);
    let b = Natural::from_limbs(&[1, 0]);
    assert_eq!(a + b, Natural::from_limbs(&[0, 0, 1]));

    let a = Natural::from_limbs(&[Limb::MAX, Limb::MAX, Limb::MAX]);
    let b = Natural::from_limbs(&[1, 0, 0]);
    assert_eq!(a + b, Natural::from_limbs(&[0, 0, 0, 1]));
  }

  #[test]
  fn test_mul_small_small() {
    assert_exprs! {
      0 * 0 = 0,
      1 * 0 = 0,
      0 * 1 = 0,
      1 * 1 = 1,
      0 * 5 = 0,
      5 * 0 = 0,
      1 * 5 = 5,
      5 * 1 = 5,
      2 * 3 = 6,
      10 * 20 = 200,
      123 * 456 = 56088
    };
  }

  #[test]
  fn test_mul_small_small_overflow() {
    assert_eq!(
      SMALL_MAX * Natural::from(2),
      Natural::from_limbs(&[0xFFFF_FFFF_FFFF_FFFE, 1])
    );

    let half_max = Natural::from(Limb::MAX / 2 + 1);
    assert_eq!(
      half_max.clone() * half_max,
      Natural::from_limbs(&[0, 0x4000_0000_0000_0000])
    );

    assert_eq!(
      SMALL_MAX * SMALL_MAX,
      Natural::from_limbs(&[0x1, 0xfffffffffffffffe])
    );
  }
}
