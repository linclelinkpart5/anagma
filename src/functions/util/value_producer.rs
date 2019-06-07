use std::collections::VecDeque;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::iter::FusedIterator;

use crate::metadata::stream::value::MetaValueStream;
use crate::metadata::types::MetaVal;
use crate::functions::Error;
use crate::functions::util::UnaryPred;
use crate::functions::util::UnaryConv;

pub enum ValueProducer<'v> {
    Source(Source<'v>),
    Fixed(Fixed<'v>),
    Raw(Raw<'v>),
    Flatten(Flatten<'v>),
    Dedup(Dedup<'v>),
    Unique(Unique<'v>),
    Filter(Filter<'v>),
    Map(Map<'v>),
    StepBy(StepBy<'v>),
    Chain(Chain<'v>),
    Zip(Zip<'v>),
    Skip(Skip<'v>),
    Take(Take<'v>),
    SkipWhile(SkipWhile<'v>),
    TakeWhile(TakeWhile<'v>),
    Intersperse(Intersperse<'v>),
    Interleave(Interleave<'v>),
}

impl<'v> From<Vec<MetaVal<'v>>> for ValueProducer<'v> {
    fn from(v: Vec<MetaVal<'v>>) -> Self {
        Self::Fixed(v.into())
    }
}

impl<'v> From<Vec<Result<MetaVal<'v>, Error>>> for ValueProducer<'v> {
    fn from(v: Vec<Result<MetaVal<'v>, Error>>) -> Self {
        Self::Raw(v.into())
    }
}

impl<'v> TryFrom<ValueProducer<'v>> for Vec<MetaVal<'v>> {
    type Error = Error;

    fn try_from(vp: ValueProducer<'v>) -> Result<Self, Self::Error> {
        vp.collect()
    }
}

impl<'v> ValueProducer<'v> {
    pub fn fixed(v: Vec<MetaVal<'v>>) -> Self {
        Self::Fixed(Fixed::new(v))
    }

    pub fn raw(v: Vec<Result<MetaVal<'v>, Error>>) -> Self {
        Self::Raw(Raw::new(v))
    }
}

impl<'v> Iterator for ValueProducer<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut Self::Source(ref mut s) => s.next(),
            &mut Self::Fixed(ref mut s) => s.next(),
            &mut Self::Raw(ref mut s) => s.next(),
            &mut Self::Flatten(ref mut s) => s.next(),
            &mut Self::Dedup(ref mut s) => s.next(),
            &mut Self::Unique(ref mut s) => s.next(),
            &mut Self::Filter(ref mut s) => s.next(),
            &mut Self::Map(ref mut s) => s.next(),
            &mut Self::StepBy(ref mut s) => s.next(),
            &mut Self::Chain(ref mut s) => s.next(),
            &mut Self::Zip(ref mut s) => s.next(),
            &mut Self::Skip(ref mut s) => s.next(),
            &mut Self::Take(ref mut s) => s.next(),
            &mut Self::SkipWhile(ref mut s) => s.next(),
            &mut Self::TakeWhile(ref mut s) => s.next(),
            &mut Self::Intersperse(ref mut s) => s.next(),
            &mut Self::Interleave(ref mut s) => s.next(),
        }
    }
}

pub struct Source<'v>(MetaValueStream<'v>);

impl<'v> Source<'v> {
    pub fn new(mvs: MetaValueStream<'v>) -> Self {
        Self(mvs)
    }
}

impl<'v> Iterator for Source<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|res| res.map(|(_, mv)| mv).map_err(Error::ValueStream))
    }
}

pub struct Fixed<'v>(std::vec::IntoIter<MetaVal<'v>>);

impl<'v> Fixed<'v> {
    pub fn new(v: Vec<MetaVal<'v>>) -> Self {
        Self(v.into_iter())
    }
}

impl<'v> Iterator for Fixed<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Result::Ok)
    }
}

impl<'v> From<Vec<MetaVal<'v>>> for Fixed<'v> {
    fn from(v: Vec<MetaVal<'v>>) -> Self {
        Fixed::new(v)
    }
}

pub struct Raw<'v>(std::vec::IntoIter<Result<MetaVal<'v>, Error>>);

impl<'v> Raw<'v> {
    pub fn new(v: Vec<Result<MetaVal<'v>, Error>>) -> Self {
        Self(v.into_iter())
    }
}

impl<'v> Iterator for Raw<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'v> From<Vec<Result<MetaVal<'v>, Error>>> for Raw<'v> {
    fn from(v: Vec<Result<MetaVal<'v>, Error>>) -> Self {
        Raw::new(v)
    }
}

pub struct Flatten<'v>(Box<ValueProducer<'v>>, VecDeque<MetaVal<'v>>);

impl<'v> Flatten<'v> {
    pub fn new(vp: ValueProducer<'v>) -> Self {
        Self(Box::new(vp), VecDeque::new())
    }
}

impl<'v> Iterator for Flatten<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.1.pop_front() {
            Some(mv) => Some(Ok(mv)),
            None => {
                // Try to get the next item from the stream.
                match self.0.next()? {
                    Ok(MetaVal::Seq(seq)) => {
                        // Move all elements in the sequence into the queue.
                        self.1.extend(seq);
                        self.next()
                    },
                    o => Some(o),
                }
            },
        }
    }
}

pub struct Dedup<'v>(Box<ValueProducer<'v>>, Option<MetaVal<'v>>);

impl<'v> Dedup<'v> {
    pub fn new(vp: ValueProducer<'v>) -> Self {
        Self(Box::new(vp), None)
    }
}

impl<'v> Iterator for Dedup<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.0.next()?;

        match res {
            Err(err) => Some(Err(err)),
            Ok(curr_val) => {
                if Some(&curr_val) != self.1.as_ref() {
                    // A non-duplicate was found.
                    self.1 = Some(curr_val.clone());
                    Some(Ok(curr_val))
                }
                else {
                    // Delegate to the next call.
                    self.next()
                }
            },
        }
    }
}

pub struct Unique<'v>(Box<ValueProducer<'v>>, HashSet<MetaVal<'v>>);

impl<'v> Unique<'v> {
    pub fn new(vp: ValueProducer<'v>) -> Self {
        Self(Box::new(vp), HashSet::new())
    }
}

impl<'v> Iterator for Unique<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.0.next()?;

        match res {
            Err(err) => Some(Err(err)),
            Ok(curr_val) => {
                if self.1.contains(&curr_val) {
                    // Skip and delegate to the next call.
                    self.next()
                }
                else {
                    self.1.insert(curr_val.clone());
                    Some(Ok(curr_val))
                }
            },
        }
    }
}

pub struct Filter<'v>(Box<ValueProducer<'v>>, UnaryPred);

impl<'v> Filter<'v> {
    pub fn new(vp: ValueProducer<'v>, pred: UnaryPred) -> Self {
        Self(Box::new(vp), pred)
    }
}

impl<'v> Iterator for Filter<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next()? {
            Ok(mv) => {
                match self.1(&mv) {
                    Err(err) => Some(Err(err)),
                    Ok(b) => {
                        if b { Some(Ok(mv)) }
                        else { self.next() }
                    },
                }
            },
            Err(err) => Some(Err(err)),
        }
    }
}

pub struct Map<'v>(Box<ValueProducer<'v>>, UnaryConv);

impl<'v> Map<'v> {
    pub fn new(vp: ValueProducer<'v>, conv: UnaryConv) -> Self {
        Self(Box::new(vp), conv)
    }
}

impl<'v> Iterator for Map<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next()? {
            Ok(mv) => Some(self.1(mv)),
            Err(err) => Some(Err(err)),
        }
    }
}

pub struct StepBy<'v> {
    vp: Box<ValueProducer<'v>>,
    curr: usize,
    n: usize,
}

impl<'v> StepBy<'v> {
    // Can fail if step size is zero.
    pub fn new(vp: ValueProducer<'v>, n: usize) -> Result<Self, Error> {
        if n == 0 { Err(Error::ZeroStepSize) }
        else {
            Ok(Self {
                vp: Box::new(vp),
                curr: n,
                n,
            })
        }
    }
}

impl<'v> Iterator for StepBy<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.vp.next()? {
            // Always report errors, even if they would not normally be "hit".
            Err(err) => Some(Err(err)),
            Ok(mv) => {
                // Output the meta value if currently at a step point.
                if self.curr >= self.n {
                    self.curr = 1;
                    Some(Ok(mv))
                }
                else {
                    self.curr += 1;
                    self.next()
                }
            }
        }
    }
}

pub struct Chain<'v>(Box<ValueProducer<'v>>, Box<ValueProducer<'v>>, bool);

impl<'v> Chain<'v> {
    pub fn new(vp_a: ValueProducer<'v>, vp_b: ValueProducer<'v>) -> Self {
        Self(Box::new(vp_a), Box::new(vp_b), false)
    }
}

impl<'v> Iterator for Chain<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        // Iterate the first stream.
        if !self.2 {
            match self.0.next() {
                None => {
                    self.2 = true;
                    self.next()
                }
                Some(res) => Some(res),
            }
        }
        // Iterate the second stream.
        else {
            self.1.next()
        }
    }
}

pub struct Zip<'v>(Box<ValueProducer<'v>>, Box<ValueProducer<'v>>);

impl<'v> Zip<'v> {
    pub fn new(vp_a: ValueProducer<'v>, vp_b: ValueProducer<'v>) -> Self {
        Self(Box::new(vp_a), Box::new(vp_b))
    }
}

impl<'v> Iterator for Zip<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let res_a = self.0.next()?;
        let res_b = self.1.next()?;

        match (res_a, res_b) {
            (Err(e_a), _) => Some(Err(e_a)),
            (_, Err(e_b)) => Some(Err(e_b)),
            (Ok(a), Ok(b)) => Some(Ok(MetaVal::Seq(vec![a, b]))),
        }
    }
}

pub struct Skip<'v> {
    vp: Box<ValueProducer<'v>>,
    curr: usize,
    n: usize,
}

impl<'v> Skip<'v> {
    pub fn new(vp: ValueProducer<'v>, n: usize) -> Self {
        Self {
            vp: Box::new(vp),
            curr: 0,
            n,
        }
    }
}

impl<'v> Iterator for Skip<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.curr < self.n {
            self.curr += 1;
            let res_mv = self.vp.next()?;

            if let Err(e) = res_mv { return Some(Err(e)) }
        }

        self.vp.next()
    }
}

pub struct Take<'v> {
    vp: Box<ValueProducer<'v>>,
    curr: usize,
    n: usize,
}

impl<'v> Take<'v> {
    pub fn new(vp: ValueProducer<'v>, n: usize) -> Self {
        Self {
            vp: Box::new(vp),
            curr: 0,
            n,
        }
    }
}

impl<'v> Iterator for Take<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr < self.n {
            self.curr += 1;
            self.vp.next()
        }
        else {
            None
        }
    }
}

pub struct SkipWhile<'v>(Box<ValueProducer<'v>>, UnaryPred, bool);

impl<'v> SkipWhile<'v> {
    pub fn new(vp: ValueProducer<'v>, u_pred: UnaryPred) -> Self {
        Self(Box::new(vp), u_pred, true)
    }
}

impl<'v> Iterator for SkipWhile<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.2 {
            loop {
                match self.0.next()? {
                    Err(e) => return Some(Err(e)),
                    Ok(mv) => {
                        match self.1(&mv) {
                            Err(e) => return Some(Err(e)),
                            Ok(true) => continue,
                            Ok(false) => {
                                self.2 = false;
                                return Some(Ok(mv))
                            }
                        }
                    },
                }
            }
        }

        self.0.next()
    }
}

pub struct TakeWhile<'v>(Box<ValueProducer<'v>>, UnaryPred, bool);

impl<'v> TakeWhile<'v> {
    pub fn new(vp: ValueProducer<'v>, u_pred: UnaryPred) -> Self {
        Self(Box::new(vp), u_pred, true)
    }
}

impl<'v> Iterator for TakeWhile<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.2 {
            match self.0.next()? {
                Ok(mv) => {
                    match self.1(&mv) {
                        Ok(true) => Some(Ok(mv)),
                        Ok(false) => {
                            self.2 = false;
                            return None
                        },
                        Err(e) => Some(Err(e)),
                    }
                },
                Err(e) => Some(Err(e)),
            }
        }
        else { None }
    }
}

pub struct Intersperse<'v>(Box<ValueProducer<'v>>, MetaVal<'v>, bool);

impl<'v> Intersperse<'v> {
    pub fn new(vp: ValueProducer<'v>, mv: MetaVal<'v>) -> Self {
        Self(Box::new(vp), mv, false)
    }
}

impl<'v> Iterator for Intersperse<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.2 = !self.2;

        if self.2 { self.0.next() }
        else { Some(Ok(self.1.clone())) }
    }
}

impl<'v> FusedIterator for Intersperse<'v> {}

pub struct Interleave<'v>(Box<ValueProducer<'v>>, Box<ValueProducer<'v>>, bool);

impl<'v> Interleave<'v> {
    pub fn new(vp_a: ValueProducer<'v>, vp_b: ValueProducer<'v>) -> Self {
        Self(Box::new(vp_a), Box::new(vp_b), false)
    }
}

impl<'v> Iterator for Interleave<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.2 = !self.2;

        if self.2 { self.0.next() }
        else { self.1.next() }
    }
}

impl<'v> FusedIterator for Interleave<'v> {}
