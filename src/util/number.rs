///! Wrapper type for items on the consumer stack that behave either as an integer or a decimal.

use std::ops::Add;
use std::ops::Sub;
use std::ops::Mul;
use std::ops::Div;
use std::ops::Rem;
use std::ops::Neg;
use std::cmp::Ordering;

use rust_decimal::Decimal;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Number {
    Integer(i64),
    Decimal(Decimal),
}

impl Number {
    /// Does a comparison based on the numerical values represented.
    /// Whole value decimals will compare as equal to their integer counterparts.
    pub fn val_cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Integer(l), Self::Integer(r)) => l.cmp(r),
            (Self::Integer(l), Self::Decimal(r)) => Decimal::from(*l).cmp(r),
            (Self::Decimal(l), Self::Integer(r)) => l.cmp(&Decimal::from(*r)),
            (Self::Decimal(l), Self::Decimal(r)) => l.cmp(r),
        }
    }

    pub fn val_eq(&self, other: &Self) -> bool {
        self.val_cmp(other) == Ordering::Equal
    }

    /// Returns the larger of two number-likes, based on their numerical values.
    /// If equal, returns the second value, to match Rust's behavior.
    pub fn val_max(self, other: Self) -> Self {
        match self.val_cmp(&other) {
            Ordering::Equal | Ordering::Less => other,
            Ordering::Greater => self,
        }
    }

    /// Returns the smaller of two number-likes, based on their numerical values.
    /// If equal, returns the first value, to match Rust's behavior.
    pub fn val_min(self, other: Self) -> Self {
        match self.val_cmp(&other) {
            Ordering::Greater => other,
            Ordering::Equal | Ordering::Less => self,
        }
    }
}

impl From<i64> for Number {
    fn from(n: i64) -> Self {
        Self::Integer(n)
    }
}

impl From<&i64> for Number {
    fn from(n: &i64) -> Self {
        Self::Integer(*n)
    }
}

impl From<Decimal> for Number {
    fn from(n: Decimal) -> Self {
        Self::Decimal(n)
    }
}

impl From<&Decimal> for Number {
    fn from(n: &Decimal) -> Self {
        Self::Decimal(*n)
    }
}

impl Add for Number {
    type Output = Number;

    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::Integer(l), Self::Integer(r)) => Self::Integer(l.add(r)),
            (Self::Integer(l), Self::Decimal(r)) => Self::Decimal(Decimal::from(l).add(r)),
            (Self::Decimal(l), Self::Integer(r)) => Self::Decimal(l.add(Decimal::from(r))),
            (Self::Decimal(l), Self::Decimal(r)) => Self::Decimal(l.add(r)),
        }
    }
}

impl Sub for Number {
    type Output = Number;

    fn sub(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::Integer(l), Self::Integer(r)) => Self::Integer(l.sub(r)),
            (Self::Integer(l), Self::Decimal(r)) => Self::Decimal(Decimal::from(l).sub(r)),
            (Self::Decimal(l), Self::Integer(r)) => Self::Decimal(l.sub(Decimal::from(r))),
            (Self::Decimal(l), Self::Decimal(r)) => Self::Decimal(l.sub(r)),
        }
    }
}

impl Mul for Number {
    type Output = Number;

    fn mul(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::Integer(l), Self::Integer(r)) => Self::Integer(l.mul(r)),
            (Self::Integer(l), Self::Decimal(r)) => Self::Decimal(Decimal::from(l).mul(r)),
            (Self::Decimal(l), Self::Integer(r)) => Self::Decimal(l.mul(Decimal::from(r))),
            (Self::Decimal(l), Self::Decimal(r)) => Self::Decimal(l.mul(r)),
        }
    }
}

impl Div for Number {
    type Output = Number;

    fn div(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::Integer(l), Self::Integer(r)) => Self::Integer(l.div(r)),
            (Self::Integer(l), Self::Decimal(r)) => Self::Decimal(Decimal::from(l).div(r)),
            (Self::Decimal(l), Self::Integer(r)) => Self::Decimal(l.div(Decimal::from(r))),
            (Self::Decimal(l), Self::Decimal(r)) => Self::Decimal(l.div(r)),
        }
    }
}

impl Rem for Number {
    type Output = Number;

    fn rem(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::Integer(l), Self::Integer(r)) => Self::Integer(l.rem(r)),
            (Self::Integer(l), Self::Decimal(r)) => Self::Decimal(Decimal::from(l).rem(r)),
            (Self::Decimal(l), Self::Integer(r)) => Self::Decimal(l.rem(Decimal::from(r))),
            (Self::Decimal(l), Self::Decimal(r)) => Self::Decimal(l.rem(r)),
        }
    }
}

impl Neg for Number {
    type Output = Number;

    // TODO: See what can be done about +/-0.0.
    fn neg(self) -> Self::Output {
        match self {
            Self::Integer(x) => Self::Integer(x.neg()),
            Self::Decimal(x) => Self::Decimal(x.neg()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cmp::Ordering;

    use rand::seq::SliceRandom;
    use rust_decimal::Decimal;

    #[test]
    fn number_val_cmp() {
        for l in -3..=3 {
            let li = Number::Integer(l);
            let ld = Number::Decimal(l.into());

            for r in -3..=3 {
                let ri = Number::Integer(r);
                let rd = Number::Decimal(r.into());

                let expected = l.cmp(&r);

                assert_eq!(expected, li.val_cmp(&ri));
                assert_eq!(expected, li.val_cmp(&rd));
                assert_eq!(expected, ld.val_cmp(&ri));
                assert_eq!(expected, ld.val_cmp(&rd));
            }

            let lower_d = Number::Decimal(Decimal::from(l) - dec!(0.5));
            let inner_i = li;
            let inner_d = ld;
            let upper_d = Number::Decimal(Decimal::from(l) + dec!(0.5));

            assert_eq!(Ordering::Greater, inner_i.val_cmp(&lower_d));
            assert_eq!(Ordering::Greater, inner_d.val_cmp(&lower_d));
            assert_eq!(Ordering::Less, inner_i.val_cmp(&upper_d));
            assert_eq!(Ordering::Less, inner_d.val_cmp(&upper_d));

            let lower_i = Number::Integer(l);
            let inner_d = upper_d;
            let upper_i = Number::Integer(l + 1);

            assert_eq!(Ordering::Greater, inner_d.val_cmp(&lower_i));
            assert_eq!(Ordering::Less, inner_d.val_cmp(&upper_i));
        }

        // Should be able to sort a list of numbers.
        let expected = vec![
            Number::Decimal(dec!(-2.5)),
            Number::Integer(-2),
            Number::Decimal(dec!(-1.5)),
            Number::Integer(-1),
            Number::Decimal(dec!(-0.5)),
            Number::Integer(0),
            Number::Decimal(dec!(0.5)),
            Number::Integer(1),
            Number::Decimal(dec!(1.5)),
            Number::Integer(2),
            Number::Decimal(dec!(2.5)),
        ];

        let mut produced = expected.clone();
        produced.shuffle(&mut rand::thread_rng());

        produced.sort_by(Number::val_cmp);

        assert_eq!(expected, produced);
    }

    #[test]
    fn number_val_eq() {
        for l in -3..=3 {
            for r in -3..=3 {
                let li = Number::Integer(l);
                let ld = Number::Decimal(l.into());
                let ri = Number::Integer(r);
                let rd = Number::Decimal(r.into());

                let expected = l.eq(&r);

                assert_eq!(expected, li.val_eq(&ri));
                assert_eq!(expected, li.val_eq(&rd));
                assert_eq!(expected, ld.val_eq(&ri));
                assert_eq!(expected, ld.val_eq(&rd));

                let ldh = Number::Decimal(Decimal::from(l) + dec!(0.5));
                let rdh = Number::Decimal(Decimal::from(r) + dec!(0.5));

                assert_eq!(expected, ldh.val_eq(&rdh));
            }
        }
    }

    #[test]
    fn number_val_max() {
        assert_eq!(Number::Integer(-1), Number::Integer(-1).val_max(Number::Integer(-2)));
        assert_eq!(Number::Integer(-1), Number::Integer(-2).val_max(Number::Integer(-1)));
        assert_eq!(Number::Integer(0), Number::Integer(0).val_max(Number::Integer(-1)));
        assert_eq!(Number::Integer(0), Number::Integer(-1).val_max(Number::Integer(0)));
        assert_eq!(Number::Integer(1), Number::Integer(1).val_max(Number::Integer(0)));
        assert_eq!(Number::Integer(1), Number::Integer(0).val_max(Number::Integer(1)));
        assert_eq!(Number::Integer(2), Number::Integer(2).val_max(Number::Integer(1)));
        assert_eq!(Number::Integer(2), Number::Integer(1).val_max(Number::Integer(2)));

        assert_eq!(Number::Decimal(dec!(-1)), Number::Decimal(dec!(-1)).val_max(Number::Decimal(dec!(-2))));
        assert_eq!(Number::Decimal(dec!(-1)), Number::Decimal(dec!(-2)).val_max(Number::Decimal(dec!(-1))));
        assert_eq!(Number::Decimal(dec!(0)), Number::Decimal(dec!(0)).val_max(Number::Decimal(dec!(-1))));
        assert_eq!(Number::Decimal(dec!(0)), Number::Decimal(dec!(-1)).val_max(Number::Decimal(dec!(0))));
        assert_eq!(Number::Decimal(dec!(1)), Number::Decimal(dec!(1)).val_max(Number::Decimal(dec!(0))));
        assert_eq!(Number::Decimal(dec!(1)), Number::Decimal(dec!(0)).val_max(Number::Decimal(dec!(1))));
        assert_eq!(Number::Decimal(dec!(2)), Number::Decimal(dec!(2)).val_max(Number::Decimal(dec!(1))));
        assert_eq!(Number::Decimal(dec!(2)), Number::Decimal(dec!(1)).val_max(Number::Decimal(dec!(2))));

        assert_eq!(Number::Decimal(dec!(1)), Number::Integer(1).val_max(Number::Decimal(dec!(1))));
        assert_eq!(Number::Integer(1), Number::Decimal(dec!(1)).val_max(Number::Integer(1)));
    }

    #[test]
    fn number_val_min() {
        assert_eq!(Number::Integer(-2), Number::Integer(-1).val_min(Number::Integer(-2)));
        assert_eq!(Number::Integer(-2), Number::Integer(-2).val_min(Number::Integer(-1)));
        assert_eq!(Number::Integer(-1), Number::Integer(0).val_min(Number::Integer(-1)));
        assert_eq!(Number::Integer(-1), Number::Integer(-1).val_min(Number::Integer(0)));
        assert_eq!(Number::Integer(0), Number::Integer(1).val_min(Number::Integer(0)));
        assert_eq!(Number::Integer(0), Number::Integer(0).val_min(Number::Integer(1)));
        assert_eq!(Number::Integer(1), Number::Integer(2).val_min(Number::Integer(1)));
        assert_eq!(Number::Integer(1), Number::Integer(1).val_min(Number::Integer(2)));

        assert_eq!(Number::Decimal(dec!(-2)), Number::Decimal(dec!(-1)).val_min(Number::Decimal(dec!(-2))));
        assert_eq!(Number::Decimal(dec!(-2)), Number::Decimal(dec!(-2)).val_min(Number::Decimal(dec!(-1))));
        assert_eq!(Number::Decimal(dec!(-1)), Number::Decimal(dec!(0)).val_min(Number::Decimal(dec!(-1))));
        assert_eq!(Number::Decimal(dec!(-1)), Number::Decimal(dec!(-1)).val_min(Number::Decimal(dec!(0))));
        assert_eq!(Number::Decimal(dec!(0)), Number::Decimal(dec!(1)).val_min(Number::Decimal(dec!(0))));
        assert_eq!(Number::Decimal(dec!(0)), Number::Decimal(dec!(0)).val_min(Number::Decimal(dec!(1))));
        assert_eq!(Number::Decimal(dec!(1)), Number::Decimal(dec!(2)).val_min(Number::Decimal(dec!(1))));
        assert_eq!(Number::Decimal(dec!(1)), Number::Decimal(dec!(1)).val_min(Number::Decimal(dec!(2))));

        assert_eq!(Number::Integer(1), Number::Integer(1).val_min(Number::Decimal(dec!(1))));
        assert_eq!(Number::Decimal(dec!(1)), Number::Decimal(dec!(1)).val_min(Number::Integer(1)));
    }

    #[test]
    fn number_from_i64() {
        for i in -3i64..=3 {
            let expected = Number::Integer(i);
            let produced = Number::from(i);

            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn number_from_decimal() {
        for i in -3i64..=3 {
            let d = Decimal::from(i) + dec!(0.5);

            let expected = Number::Decimal(d);
            let produced = Number::from(d);

            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn number_add() {
        for l in -3..=3 {
            for r in -3..=3 {
                let li = Number::Integer(l);
                let ld = Number::Decimal(l.into());
                let ri = Number::Integer(r);
                let rd = Number::Decimal(r.into());

                let raw = l.add(r);
                let expected_i = Number::from(raw);
                let expected_d = Number::from(Decimal::from(raw));

                assert_eq!(expected_i, li.add(ri));
                assert_eq!(expected_d, li.add(rd));
                assert_eq!(expected_d, ld.add(ri));
                assert_eq!(expected_d, ld.add(rd));
            }
        }

        let input_a_pos = Number::Decimal(dec!(3.2));
        let input_b_pos = Number::Decimal(dec!(1.8));
        let input_a_neg = Number::Decimal(dec!(-3.2));
        let input_b_neg = Number::Decimal(dec!(-1.8));

        let expected_a_pos_b_pos = Number::Decimal(dec!(5.0));
        let expected_a_pos_b_neg = Number::Decimal(dec!(1.4));
        let expected_a_neg_b_pos = Number::Decimal(dec!(-1.4));
        let expected_a_neg_b_neg = Number::Decimal(dec!(-5.0));
        let expected_b_pos_a_pos = expected_a_pos_b_pos;
        let expected_b_pos_a_neg = expected_a_neg_b_pos;
        let expected_b_neg_a_pos = expected_a_pos_b_neg;
        let expected_b_neg_a_neg = expected_a_neg_b_neg;

        assert_eq!(expected_a_pos_b_pos, input_a_pos.add(input_b_pos));
        assert_eq!(expected_a_pos_b_neg, input_a_pos.add(input_b_neg));
        assert_eq!(expected_a_neg_b_pos, input_a_neg.add(input_b_pos));
        assert_eq!(expected_a_neg_b_neg, input_a_neg.add(input_b_neg));
        assert_eq!(expected_b_pos_a_pos, input_b_pos.add(input_a_pos));
        assert_eq!(expected_b_pos_a_neg, input_b_pos.add(input_a_neg));
        assert_eq!(expected_b_neg_a_pos, input_b_neg.add(input_a_pos));
        assert_eq!(expected_b_neg_a_neg, input_b_neg.add(input_a_neg));
    }

    #[test]
    fn number_sub() {
        for l in -3..=3 {
            for r in -3..=3 {
                let li = Number::Integer(l);
                let ld = Number::Decimal(l.into());
                let ri = Number::Integer(r);
                let rd = Number::Decimal(r.into());

                let raw = l.sub(r);
                let expected_i = Number::from(raw);
                let expected_d = Number::from(Decimal::from(raw));

                assert_eq!(expected_i, li.sub(ri));
                assert_eq!(expected_d, li.sub(rd));
                assert_eq!(expected_d, ld.sub(ri));
                assert_eq!(expected_d, ld.sub(rd));
            }
        }

        let input_a_pos = Number::Decimal(dec!(3.2));
        let input_b_pos = Number::Decimal(dec!(1.8));
        let input_a_neg = Number::Decimal(dec!(-3.2));
        let input_b_neg = Number::Decimal(dec!(-1.8));

        let expected_a_pos_b_pos = Number::Decimal(dec!(1.4));
        let expected_a_pos_b_neg = Number::Decimal(dec!(5.0));
        let expected_a_neg_b_pos = Number::Decimal(dec!(-5.0));
        let expected_a_neg_b_neg = Number::Decimal(dec!(-1.4));
        let expected_b_pos_a_pos = -expected_a_pos_b_pos;
        let expected_b_pos_a_neg = -expected_a_neg_b_pos;
        let expected_b_neg_a_pos = -expected_a_pos_b_neg;
        let expected_b_neg_a_neg = -expected_a_neg_b_neg;

        assert_eq!(expected_a_pos_b_pos, input_a_pos.sub(input_b_pos));
        assert_eq!(expected_a_pos_b_neg, input_a_pos.sub(input_b_neg));
        assert_eq!(expected_a_neg_b_pos, input_a_neg.sub(input_b_pos));
        assert_eq!(expected_a_neg_b_neg, input_a_neg.sub(input_b_neg));
        assert_eq!(expected_b_pos_a_pos, input_b_pos.sub(input_a_pos));
        assert_eq!(expected_b_pos_a_neg, input_b_pos.sub(input_a_neg));
        assert_eq!(expected_b_neg_a_pos, input_b_neg.sub(input_a_pos));
        assert_eq!(expected_b_neg_a_neg, input_b_neg.sub(input_a_neg));
    }

    #[test]
    fn number_mul() {
        for l in -3..=3 {
            for r in -3..=3 {
                let li = Number::Integer(l);
                let ld = Number::Decimal(l.into());
                let ri = Number::Integer(r);
                let rd = Number::Decimal(r.into());

                let raw = l.mul(r);
                let expected_i = Number::from(raw);
                let expected_d = Number::from(Decimal::from(raw));

                assert_eq!(expected_i, li.mul(ri));
                assert_eq!(expected_d, li.mul(rd));
                assert_eq!(expected_d, ld.mul(ri));
                assert_eq!(expected_d, ld.mul(rd));
            }
        }

        let input_a_pos = Number::Decimal(dec!(3.2));
        let input_b_pos = Number::Decimal(dec!(1.8));
        let input_a_neg = Number::Decimal(dec!(-3.2));
        let input_b_neg = Number::Decimal(dec!(-1.8));

        let expected_a_pos_b_pos = Number::Decimal(dec!(5.76));
        let expected_a_pos_b_neg = Number::Decimal(dec!(-5.76));
        let expected_a_neg_b_pos = Number::Decimal(dec!(-5.76));
        let expected_a_neg_b_neg = Number::Decimal(dec!(5.76));
        let expected_b_pos_a_pos = expected_a_pos_b_pos;
        let expected_b_pos_a_neg = expected_a_neg_b_pos;
        let expected_b_neg_a_pos = expected_a_pos_b_neg;
        let expected_b_neg_a_neg = expected_a_neg_b_neg;

        assert_eq!(expected_a_pos_b_pos, input_a_pos.mul(input_b_pos));
        assert_eq!(expected_a_pos_b_neg, input_a_pos.mul(input_b_neg));
        assert_eq!(expected_a_neg_b_pos, input_a_neg.mul(input_b_pos));
        assert_eq!(expected_a_neg_b_neg, input_a_neg.mul(input_b_neg));
        assert_eq!(expected_b_pos_a_pos, input_b_pos.mul(input_a_pos));
        assert_eq!(expected_b_pos_a_neg, input_b_pos.mul(input_a_neg));
        assert_eq!(expected_b_neg_a_pos, input_b_neg.mul(input_a_pos));
        assert_eq!(expected_b_neg_a_neg, input_b_neg.mul(input_a_neg));
    }

    #[test]
    fn number_div() {
        for l in -3..=3 {
            for r in -3..=3 {
                let li = Number::Integer(l);
                let ld = Number::Decimal(l.into());
                let ri = Number::Integer(r);
                let rd = Number::Decimal(r.into());

                if r == 0 {
                    assert!(std::panic::catch_unwind(|| li.div(ri)).is_err());
                    assert!(std::panic::catch_unwind(|| li.div(rd)).is_err());
                    assert!(std::panic::catch_unwind(|| ld.div(ri)).is_err());
                    assert!(std::panic::catch_unwind(|| ld.div(rd)).is_err());
                }
                else {
                    let expected_i = Number::from(l.div(r));
                    let expected_d = Number::from(Decimal::from(l).div(Decimal::from(r)));

                    assert_eq!(expected_i, li.div(ri));
                    assert_eq!(expected_d, li.div(rd));
                    assert_eq!(expected_d, ld.div(ri));
                    assert_eq!(expected_d, ld.div(rd));
                }
            }
        }

        let input_a_pos = Number::Decimal(dec!(3.2));
        let input_b_pos = Number::Decimal(dec!(1.6));
        let input_a_neg = Number::Decimal(dec!(-3.2));
        let input_b_neg = Number::Decimal(dec!(-1.6));

        let expected_a_pos_b_pos = Number::Decimal(dec!(2.0));
        let expected_a_pos_b_neg = Number::Decimal(dec!(-2.0));
        let expected_a_neg_b_pos = Number::Decimal(dec!(-2.0));
        let expected_a_neg_b_neg = Number::Decimal(dec!(2.0));
        let expected_b_pos_a_pos = Number::Decimal(dec!(0.5));
        let expected_b_pos_a_neg = Number::Decimal(dec!(-0.5));
        let expected_b_neg_a_pos = Number::Decimal(dec!(-0.5));
        let expected_b_neg_a_neg = Number::Decimal(dec!(0.5));

        assert_eq!(expected_a_pos_b_pos, input_a_pos.div(input_b_pos));
        assert_eq!(expected_a_pos_b_neg, input_a_pos.div(input_b_neg));
        assert_eq!(expected_a_neg_b_pos, input_a_neg.div(input_b_pos));
        assert_eq!(expected_a_neg_b_neg, input_a_neg.div(input_b_neg));
        assert_eq!(expected_b_pos_a_pos, input_b_pos.div(input_a_pos));
        assert_eq!(expected_b_pos_a_neg, input_b_pos.div(input_a_neg));
        assert_eq!(expected_b_neg_a_pos, input_b_neg.div(input_a_pos));
        assert_eq!(expected_b_neg_a_neg, input_b_neg.div(input_a_neg));
    }

    #[test]
    fn number_rem() {
        for l in -3..=3 {
            for r in -3..=3 {
                let li = Number::Integer(l);
                let ld = Number::Decimal(l.into());
                let ri = Number::Integer(r);
                let rd = Number::Decimal(r.into());

                if r == 0 {
                    assert!(std::panic::catch_unwind(|| li.rem(ri)).is_err());
                    assert!(std::panic::catch_unwind(|| li.rem(rd)).is_err());
                    assert!(std::panic::catch_unwind(|| ld.rem(ri)).is_err());
                    assert!(std::panic::catch_unwind(|| ld.rem(rd)).is_err());
                }
                else {
                    let expected_i = Number::from(l.rem(r));
                    let expected_d = Number::from(Decimal::from(l).rem(Decimal::from(r)));

                    assert_eq!(expected_i, li.rem(ri));
                    assert_eq!(expected_d, li.rem(rd));
                    assert_eq!(expected_d, ld.rem(ri));
                    assert_eq!(expected_d, ld.rem(rd));
                }
            }
        }

        let input_a_pos = Number::Decimal(dec!(3.2));
        let input_b_pos = Number::Decimal(dec!(1.6));
        let input_a_neg = Number::Decimal(dec!(-3.2));
        let input_b_neg = Number::Decimal(dec!(-1.6));

        let expected_a_pos_b_pos = Number::Decimal(dec!(0.0));
        let expected_a_pos_b_neg = Number::Decimal(dec!(0.0));
        let expected_a_neg_b_pos = Number::Decimal(dec!(-0.0));
        let expected_a_neg_b_neg = Number::Decimal(dec!(-0.0));
        let expected_b_pos_a_pos = Number::Decimal(dec!(1.6));
        let expected_b_pos_a_neg = Number::Decimal(dec!(1.6));
        let expected_b_neg_a_pos = Number::Decimal(dec!(-1.6));
        let expected_b_neg_a_neg = Number::Decimal(dec!(-1.6));

        assert_eq!(expected_a_pos_b_pos, input_a_pos.rem(input_b_pos));
        assert_eq!(expected_a_pos_b_neg, input_a_pos.rem(input_b_neg));
        assert_eq!(expected_a_neg_b_pos, input_a_neg.rem(input_b_pos));
        assert_eq!(expected_a_neg_b_neg, input_a_neg.rem(input_b_neg));
        assert_eq!(expected_b_pos_a_pos, input_b_pos.rem(input_a_pos));
        assert_eq!(expected_b_pos_a_neg, input_b_pos.rem(input_a_neg));
        assert_eq!(expected_b_neg_a_pos, input_b_neg.rem(input_a_pos));
        assert_eq!(expected_b_neg_a_neg, input_b_neg.rem(input_a_neg));
    }

    #[test]
    fn number_neg() {
        for x in -3..=3 {
            let xi = Number::Integer(x);
            let xd = Number::Decimal(x.into());

            let expected_i = Number::from(-x);
            let expected_d = Number::from(-Decimal::from(x));

            assert_eq!(expected_i, xi.neg());
            assert_eq!(expected_d, xd.neg());
        }

        let input_pos = Number::Decimal(dec!(3.2));
        let input_neg = Number::Decimal(dec!(-3.2));

        let expected_pos = input_neg;
        let expected_neg = input_pos;

        assert_eq!(expected_pos, input_pos.neg());
        assert_eq!(expected_neg, input_neg.neg());
    }
}
