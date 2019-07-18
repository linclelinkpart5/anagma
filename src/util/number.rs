///! Wrapper type for items on the consumer stack that behave either as an integer or a decimal.

use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Mul;
use std::ops::MulAssign;
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

impl From<Decimal> for Number {
    fn from(n: Decimal) -> Self {
        Self::Decimal(n)
    }
}

impl Add for Number {
    type Output = Number;

    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::Integer(l), Self::Integer(r)) => Self::Integer(l + r),
            (Self::Integer(l), Self::Decimal(r)) => Self::Decimal(Decimal::from(l) + r),
            (Self::Decimal(l), Self::Integer(r)) => Self::Decimal(l + Decimal::from(r)),
            (Self::Decimal(l), Self::Decimal(r)) => Self::Decimal(l + r),
        }
    }
}

impl AddAssign for Number {
    fn add_assign(&mut self, other: Self) {
        *self = match (&self, other) {
            (&Self::Integer(ref l), Self::Integer(r)) => Self::Integer(l + r),
            (&Self::Integer(ref l), Self::Decimal(ref r)) => Self::Decimal(Decimal::from(*l) + r),
            (&Self::Decimal(ref l), Self::Integer(r)) => Self::Decimal(l + Decimal::from(r)),
            (&Self::Decimal(ref l), Self::Decimal(ref r)) => Self::Decimal(l + r),
        };
    }
}

impl Mul for Number {
    type Output = Number;

    fn mul(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::Integer(l), Self::Integer(r)) => Self::Integer(l * r),
            (Self::Integer(l), Self::Decimal(r)) => Self::Decimal(Decimal::from(l) * r),
            (Self::Decimal(l), Self::Integer(r)) => Self::Decimal(l * Decimal::from(r)),
            (Self::Decimal(l), Self::Decimal(r)) => Self::Decimal(l * r),
        }
    }
}

impl MulAssign for Number {
    fn mul_assign(&mut self, other: Self) {
        *self = match (&self, other) {
            (&Self::Integer(ref l), Self::Integer(r)) => Self::Integer(l * r),
            (&Self::Integer(ref l), Self::Decimal(ref r)) => Self::Decimal(Decimal::from(*l) * r),
            (&Self::Decimal(ref l), Self::Integer(r)) => Self::Decimal(l * Decimal::from(r)),
            (&Self::Decimal(ref l), Self::Decimal(ref r)) => Self::Decimal(l * r),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::Number as NL;

    use std::cmp::Ordering;

    use rand::seq::SliceRandom;
    use rust_decimal::Decimal;

    #[test]
    fn number_val_cmp() {
        for l in -3..=3 {
            let li = NL::Integer(l);
            let ld = NL::Decimal(l.into());

            for r in -3..=3 {
                let ri = NL::Integer(r);
                let rd = NL::Decimal(r.into());

                let expected = l.cmp(&r);

                assert_eq!(expected, li.val_cmp(&ri));
                assert_eq!(expected, li.val_cmp(&rd));
                assert_eq!(expected, ld.val_cmp(&ri));
                assert_eq!(expected, ld.val_cmp(&rd));
            }

            let lower_d = NL::Decimal(Decimal::from(l) - dec!(0.5));
            let inner_i = li;
            let inner_d = ld;
            let upper_d = NL::Decimal(Decimal::from(l) + dec!(0.5));

            assert_eq!(Ordering::Greater, inner_i.val_cmp(&lower_d));
            assert_eq!(Ordering::Greater, inner_d.val_cmp(&lower_d));
            assert_eq!(Ordering::Less, inner_i.val_cmp(&upper_d));
            assert_eq!(Ordering::Less, inner_d.val_cmp(&upper_d));

            let lower_i = NL::Integer(l);
            let inner_d = upper_d;
            let upper_i = NL::Integer(l + 1);

            assert_eq!(Ordering::Greater, inner_d.val_cmp(&lower_i));
            assert_eq!(Ordering::Less, inner_d.val_cmp(&upper_i));
        }

        // Should be able to sort a list of numbers.
        let expected = vec![
            NL::Decimal(dec!(-2.5)),
            NL::Integer(-2),
            NL::Decimal(dec!(-1.5)),
            NL::Integer(-1),
            NL::Decimal(dec!(-0.5)),
            NL::Integer(0),
            NL::Decimal(dec!(0.5)),
            NL::Integer(1),
            NL::Decimal(dec!(1.5)),
            NL::Integer(2),
            NL::Decimal(dec!(2.5)),
        ];

        let mut produced = expected.clone();
        produced.shuffle(&mut rand::thread_rng());

        produced.sort_by(NL::val_cmp);

        assert_eq!(expected, produced);
    }

    #[test]
    fn number_val_eq() {
        for l in -3..=3 {
            for r in -3..=3 {
                let li = NL::Integer(l);
                let ld = NL::Decimal(l.into());
                let ri = NL::Integer(r);
                let rd = NL::Decimal(r.into());

                let expected = l.eq(&r);

                assert_eq!(expected, li.val_eq(&ri));
                assert_eq!(expected, li.val_eq(&rd));
                assert_eq!(expected, ld.val_eq(&ri));
                assert_eq!(expected, ld.val_eq(&rd));

                let ldh = NL::Decimal(Decimal::from(l) + dec!(0.5));
                let rdh = NL::Decimal(Decimal::from(r) + dec!(0.5));

                assert_eq!(expected, ldh.val_eq(&rdh));
            }
        }
    }
}
