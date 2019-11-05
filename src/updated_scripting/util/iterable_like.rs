
use std::borrow::Cow;
use std::convert::TryInto;
use std::convert::TryFrom;

use crate::util::Number;
use crate::metadata::types::MetaVal;
use crate::updated_scripting::Error;
use crate::updated_scripting::arg::Arg;
use crate::updated_scripting::util::Util;
use crate::updated_scripting::util::IteratorLike;
use crate::updated_scripting::util::Producer;
use crate::updated_scripting::util::producer::Fixed;
use crate::updated_scripting::util::producer::Flatten;
use crate::updated_scripting::util::producer::Dedup;
use crate::updated_scripting::util::producer::Unique;
use crate::updated_scripting::util::producer::Filter;
use crate::updated_scripting::util::producer::Map;
use crate::updated_scripting::util::producer::StepBy;
use crate::updated_scripting::util::producer::Chain;
use crate::updated_scripting::util::producer::Zip;
use crate::updated_scripting::util::producer::Skip;
use crate::updated_scripting::util::producer::Take;
use crate::updated_scripting::util::producer::SkipWhile;
use crate::updated_scripting::util::producer::TakeWhile;
use crate::updated_scripting::util::producer::Intersperse;
use crate::updated_scripting::util::producer::RoundRobin;
use crate::updated_scripting::ops::Predicate;
use crate::updated_scripting::ops::Converter;

#[derive(Copy, Clone)]
enum RevSort { Rev, Sort, }

#[derive(Clone, Copy)]
enum MinMax { Min, Max, }

#[derive(Clone, Copy)]
enum SumProd { Sum, Prod, }

#[derive(Clone, Copy)]
enum AllAny { All, Any, }

/// Represents one of several different kinds of iterables, producing meta values.
pub enum IterableLike<'a> {
    Slice(&'a [MetaVal]),
    Vector(Vec<MetaVal>),
    Producer(Producer),
}

impl<'a> IntoIterator for IterableLike<'a> {
    type Item = Result<Cow<'a, MetaVal>, Error>;
    type IntoIter = IteratorLike<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Slice(s) => IteratorLike::Slice(s.into_iter()),
            Self::Vector(v) => IteratorLike::Vector(v.into_iter()),
            Self::Producer(p) => IteratorLike::Producer(p),
        }
    }
}

impl<'a> TryFrom<MetaVal> for IterableLike<'a> {
    type Error = Error;

    fn try_from(mv: MetaVal) -> Result<Self, Self::Error> {
        match mv {
            MetaVal::Seq(v) => Ok(Self::Vector(v)),
            _ => Err(Error::NotSequence),
        }
    }
}

impl<'a> TryFrom<&'a MetaVal> for IterableLike<'a> {
    type Error = Error;

    fn try_from(mv: &'a MetaVal) -> Result<Self, Self::Error> {
        match mv {
            &MetaVal::Seq(ref v) => Ok(Self::Slice(v.as_ref())),
            _ => Err(Error::NotSequence),
        }
    }
}

impl<'a> TryFrom<Arg> for IterableLike<'a> {
    type Error = Error;

    fn try_from(arg: Arg) -> Result<Self, Self::Error> {
        match arg {
            Arg::Value(mv) => mv.try_into(),
            _ => Err(Error::NotSequence),
        }
    }
}

impl<'a> TryFrom<&'a Arg> for IterableLike<'a> {
    type Error = Error;

    fn try_from(arg: &'a Arg) -> Result<Self, Self::Error> {
        match arg {
            &Arg::Value(ref mv) => mv.try_into(),
            _ => Err(Error::NotSequence),
        }
    }
}

impl<'a> IterableLike<'a> {
    fn is_lazy(&self) -> bool {
        match self {
            &Self::Slice(..) => false,
            &Self::Vector(..) => false,
            &Self::Producer(..) => true,
        }
    }

    fn into_producer(self) -> (Producer, bool) {
        let is_lazy = self.is_lazy();

        let producer = match self {
            Self::Slice(s) => Producer::new(Fixed::new(s.to_vec())),
            Self::Vector(v) => Producer::new(Fixed::new(v)),
            Self::Producer(p) => p,
        };

        (producer, is_lazy)
    }

    /// Collects the contained items eagerly.
    /// This is a no-op if this iterable is already collected.
    pub fn collect(self) -> Result<Vec<MetaVal>, Error> {
        match self {
            Self::Slice(s) => Ok(s.to_vec()),
            Self::Vector(v) => Ok(v),
            Self::Producer(p) => p.collect(),
        }
    }

    /// Helper method for `rev`/`sort`.
    fn rev_sort(self, flag: RevSort) -> Result<Vec<MetaVal>, Error> {
        let mut seq = self.collect()?;

        match flag {
            // Reverse the slice mutably in-place.
            RevSort::Rev => seq.reverse(),

            // Sort the slice mutably in-place, using the default sort comparison.
            RevSort::Sort => seq.sort_by(Util::default_sort_by),
        };

        Ok(seq)
    }

    /// Reverses the order of the items in this iterable.
    /// Eagerly collects the items beforehand if not already collected.
    pub fn rev(self) -> Result<Vec<MetaVal>, Error> {
        self.rev_sort(RevSort::Rev)
    }

    /// Sorts the items in this iterable using the default sort comparison.
    /// Eagerly collects the items beforehand if not already collected.
    pub fn sort(self) -> Result<Vec<MetaVal>, Error> {
        self.rev_sort(RevSort::Sort)
    }

    /// Counts the number of items contained in this iterable.
    pub fn count(self) -> Result<usize, Error> {
        match self {
            Self::Slice(s) => Ok(s.len()),
            Self::Vector(v) => Ok(v.len()),
            Self::Producer(p) => p.count(),
        }
    }

    /// Returns the first item in this iterable, if there is one.
    pub fn first(self) -> Result<Option<Cow<'a, MetaVal>>, Error> {
        match self {
            Self::Slice(s) => Ok(s.first().map(Cow::Borrowed)),
            Self::Vector(v) => Ok(v.into_iter().next().map(Cow::Owned)),
            Self::Producer(p) => Ok(p.first()?.map(Cow::Owned)),
        }
    }

    /// Returns the last item in this iterable, if there is one.
    pub fn last(self) -> Result<Option<Cow<'a, MetaVal>>, Error> {
        match self {
            Self::Slice(s) => Ok(s.last().map(Cow::Borrowed)),
            Self::Vector(v) => Ok(v.into_iter().last().map(Cow::Owned)),
            Self::Producer(p) => Ok(p.last()?.map(Cow::Owned)),
        }
    }

    /// Helper method for `min_in`/`max_in`.
    fn min_in_max_in(self, flag: MinMax) -> Result<Option<Number>, Error> {
        let mut it = self.into_iter();
        match it.next() {
            // No items, so no min or max.
            None => Ok(None),

            Some(res_first_item) => {
                let first_item = res_first_item?;
                let mut target_num: Number = first_item.as_ref().try_into().map_err(|_| Error::NotNumeric)?;

                for res_item in it {
                    let item = res_item?;
                    let num: Number = item.as_ref().try_into().map_err(|_| Error::NotNumeric)?;
                    target_num = match flag {
                        MinMax::Min => target_num.val_min(num),
                        MinMax::Max => target_num.val_max(num),
                    };
                }

                Ok(Some(target_num))
            }
        }
    }

    /// Returns the minimum number in this iterable, using the default numerical comparison.
    /// Returns an error if any non-numeric items are found.
    pub fn min_in(self) -> Result<Option<Number>, Error> {
        self.min_in_max_in(MinMax::Min)
    }

    /// Returns the maximum number in this iterable, using the default numerical comparison.
    /// Returns an error if any non-numeric items are found.
    pub fn max_in(self) -> Result<Option<Number>, Error> {
        self.min_in_max_in(MinMax::Max)
    }

    /// Helper method for `sum`/`prod`.
    fn sum_prod(self, flag: SumProd) -> Result<Number, Error> {
        // Additive/multiplicative identites.
        let mut total = match flag {
            SumProd::Sum => Number::Integer(0),
            SumProd::Prod => Number::Integer(1),
        };

        for res_item in self {
            let item = res_item?;
            let num: Number = item.as_ref().try_into().map_err(|_| Error::NotNumeric)?;

            match flag {
                SumProd::Sum => { total = total + num; },
                SumProd::Prod => { total = total * num; },
            };
        }

        Ok(total)
    }

    /// Sums the numbers in this iterable.
    /// Returns an error if any non-numeric items are found.
    pub fn sum(self) -> Result<Number, Error> {
        self.sum_prod(SumProd::Sum)
    }

    /// Multiplies the numbers in this iterable.
    /// Returns an error if any non-numeric items are found.
    pub fn prod(self) -> Result<Number, Error> {
        self.sum_prod(SumProd::Prod)
    }

    /// Checks if all items are equal to each other.
    /// If empty, returns true.
    pub fn all_equal(self) -> Result<bool, Error> {
        let mut it = self.into_iter();
        match it.next() {
            None => Ok(true),
            Some(res_first_item) => {
                let first_item = res_first_item?;
                for res_item in it { if res_item? != first_item { return Ok(false) } }
                Ok(true)
            },
        }
    }

    /// Checks if the iterable has no items.
    /// If empty, returns true.
    pub fn is_empty(self) -> Result<bool, Error> {
        match self {
            Self::Slice(s) => Ok(s.is_empty()),
            Self::Vector(v) => Ok(v.is_empty()),
            Self::Producer(p) => p.is_empty(),
        }
    }

    /// Produces a new iterable with one level of nested sub-sequences flattened out.
    pub fn flatten(self) -> Result<Self, Error> {
        let (inner, is_lazy) = self.into_producer();

        let producer = Flatten::new(inner);

        if is_lazy { Ok(Self::Producer(Producer::new(producer))) }
        else { Ok(Self::Vector(producer.collect::<Result<Vec<_>, _>>()?)) }
    }

    /// Produces a new iterable with consecutive duplicated items removed.
    pub fn dedup(self) -> Result<Self, Error> {
        let (inner, is_lazy) = self.into_producer();

        let producer = Dedup::new(inner);

        if is_lazy { Ok(Self::Producer(Producer::new(producer))) }
        else { Ok(Self::Vector(producer.collect::<Result<Vec<_>, _>>()?)) }
    }

    /// Produces a new iterable with only unique items.
    pub fn unique(self) -> Result<Self, Error> {
        let (inner, is_lazy) = self.into_producer();

        let producer = Unique::new(inner);

        if is_lazy { Ok(Self::Producer(Producer::new(producer))) }
        else { Ok(Self::Vector(producer.collect::<Result<Vec<_>, _>>()?)) }
    }

    /// Returns the item at a specific index position in the iterable, if present.
    pub fn nth(self, n: usize) -> Result<Option<Cow<'a, MetaVal>>, Error> {
        match self {
            Self::Slice(s) => Ok(s.get(n).map(Cow::Borrowed)),
            Self::Vector(v) => Ok(v.into_iter().nth(n).map(Cow::Owned)),
            Self::Producer(p) => Ok(p.nth(n)?.map(Cow::Owned)),
        }
    }

    /// Helper method for `all`/`any`.
    fn all_any(self, pred: Predicate, flag: AllAny) -> Result<bool, Error> {
        let target = match flag {
            AllAny::All => false,
            AllAny::Any => true,
        };

        for res_item in self {
            let item = res_item?;
            if pred.test(&item)? == target { return Ok(target) }
        }

        Ok(!target)
    }

    /// Applies a predicate to each item in the iterable.
    /// Returns false if the iterable contains an item for which the predicate returns false.
    pub fn all(self, pred: Predicate) -> Result<bool, Error> {
        self.all_any(pred, AllAny::All)
    }

    /// Applies a predicate to each item in the iterable.
    /// Returns true if the iterable contains an item for which the predicate returns true.
    pub fn any(self, pred: Predicate) -> Result<bool, Error> {
        self.all_any(pred, AllAny::Any)
    }

    /// Helper method for `find`/`position`.
    fn find_position(self, pred: Predicate) -> Result<Option<(usize, Cow<'a, MetaVal>)>, Error> {
        for (n, res_item) in self.into_iter().enumerate() {
            let item = res_item?;
            if pred.test(&item)? { return Ok(Some((n, item))) }
        }

        Ok(None)
    }

    /// Finds the first item in the iterable that passes a predicate, and returns the item.
    /// If no items pass the predicate, returns `None`.
    pub fn find(self, pred: Predicate) -> Result<Option<Cow<'a, MetaVal>>, Error> {
        Ok(self.find_position(pred)?.map(|(_, item)| item))
    }

    /// Finds the first item in the iterable that passes a predicate, and returns the index of the item.
    /// If no items pass the predicate, returns `None`.
    pub fn position(self, pred: Predicate) -> Result<Option<usize>, Error> {
        Ok(self.find_position(pred)?.map(|(index, _)| index))
    }

    /// Produces a new iterable containing only items that pass a given predicate.
    pub fn filter(self, pred: Predicate) -> Result<Self, Error> {
        let (inner, is_lazy) = self.into_producer();

        let producer = Filter::new(inner, pred);

        if is_lazy { Ok(Self::Producer(Producer::new(producer))) }
        else { Ok(Self::Vector(producer.collect::<Result<Vec<_>, _>>()?)) }
    }

    /// Produces a new iterable by applying a converter to each item in the original iterable.
    pub fn map(self, conv: Converter) -> Result<Self, Error> {
        let (inner, is_lazy) = self.into_producer();

        let producer = Map::new(inner, conv);

        if is_lazy { Ok(Self::Producer(Producer::new(producer))) }
        else { Ok(Self::Vector(producer.collect::<Result<Vec<_>, _>>()?)) }
    }

    /// Produces a new iterable by skipping a fixed number of items from the original iterable after each item.
    pub fn step_by(self, step: usize) -> Result<Self, Error> {
        let (inner, is_lazy) = self.into_producer();

        let producer = StepBy::new(inner, step);

        if is_lazy { Ok(Self::Producer(Producer::new(producer))) }
        else { Ok(Self::Vector(producer.collect::<Result<Vec<_>, _>>()?)) }
    }

    /// Produces a new iterable by concatenating ("chaining") together this iterable with another.
    pub fn chain(self, iter: Self) -> Result<Self, Error> {
        let (inner_a, is_lazy_a) = self.into_producer();
        let (inner_b, is_lazy_b) = iter.into_producer();
        let is_lazy = is_lazy_a || is_lazy_b;

        let producer = Chain::new(inner_a, inner_b);

        if is_lazy { Ok(Self::Producer(Producer::new(producer))) }
        else { Ok(Self::Vector(producer.collect::<Result<Vec<_>, _>>()?)) }
    }

    /// Produces a new iterable that yields pairs of items from this and another iterable.
    /// Stops when the shorter of the two iterables is exhausted.
    pub fn zip(self, iter: Self) -> Result<Self, Error> {
        let (inner_a, is_lazy_a) = self.into_producer();
        let (inner_b, is_lazy_b) = iter.into_producer();
        let is_lazy = is_lazy_a || is_lazy_b;

        let producer = Zip::new(inner_a, inner_b);

        if is_lazy { Ok(Self::Producer(Producer::new(producer))) }
        else { Ok(Self::Vector(producer.collect::<Result<Vec<_>, _>>()?)) }
    }

    /// Produces a new iterable that skips a fixed number of items from the start.
    pub fn skip(self, n: usize) -> Result<Self, Error> {
        let (inner, is_lazy) = self.into_producer();

        let producer = Skip::new(inner, n);

        if is_lazy { Ok(Self::Producer(Producer::new(producer))) }
        else { Ok(Self::Vector(producer.collect::<Result<Vec<_>, _>>()?)) }
    }

    /// Produces a new iterable that takes a fixed number of items from the start.
    pub fn take(self, n: usize) -> Result<Self, Error> {
        let (inner, is_lazy) = self.into_producer();

        let producer = Take::new(inner, n);

        if is_lazy { Ok(Self::Producer(Producer::new(producer))) }
        else { Ok(Self::Vector(producer.collect::<Result<Vec<_>, _>>()?)) }
    }

    /// Produces a new iterable that skips items from the start while a predicate returns true.
    pub fn skip_while(self, pred: Predicate) -> Result<Self, Error> {
        let (inner, is_lazy) = self.into_producer();

        let producer = SkipWhile::new(inner, pred);

        if is_lazy { Ok(Self::Producer(Producer::new(producer))) }
        else { Ok(Self::Vector(producer.collect::<Result<Vec<_>, _>>()?)) }
    }

    /// Produces a new iterable that takes items from the start while a predicate returns true.
    pub fn take_while(self, pred: Predicate) -> Result<Self, Error> {
        let (inner, is_lazy) = self.into_producer();

        let producer = TakeWhile::new(inner, pred);

        if is_lazy { Ok(Self::Producer(Producer::new(producer))) }
        else { Ok(Self::Vector(producer.collect::<Result<Vec<_>, _>>()?)) }
    }

    /// Produces a new iterable that alternates between items from this iterable and a constant item.
    pub fn intersperse(self, item: MetaVal) -> Result<Self, Error> {
        let (inner, is_lazy) = self.into_producer();

        let producer = Intersperse::new(inner, item);

        if is_lazy { Ok(Self::Producer(Producer::new(producer))) }
        else { Ok(Self::Vector(producer.collect::<Result<Vec<_>, _>>()?)) }
    }

    /// Produces a new iterable that alternates between items from this iterable and another.
    /// If one iterable runs out before the other, the items from the remaining iterable are yielded.
    pub fn round_robin(self, iter: Self) -> Result<Self, Error> {
        let (inner_a, is_lazy_a) = self.into_producer();
        let (inner_b, is_lazy_b) = iter.into_producer();
        let is_lazy = is_lazy_a || is_lazy_b;

        let producer = RoundRobin::new(inner_a, inner_b);

        if is_lazy { Ok(Self::Producer(Producer::new(producer))) }
        else { Ok(Self::Vector(producer.collect::<Result<Vec<_>, _>>()?)) }
    }
}
