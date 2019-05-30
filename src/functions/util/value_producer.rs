use std::collections::VecDeque;
use std::collections::HashSet;
use std::iter::FusedIterator;

use crate::metadata::stream::value::MetaValueStream;
use crate::metadata::types::MetaVal;
use crate::functions::Error;
use crate::functions::util::UnaryPred;
use crate::functions::util::UnaryConv;

pub trait ValueProducer<'v>: Iterator<Item = Result<MetaVal<'v>, Error>> {}

pub struct Source<'v>(MetaValueStream<'v>);

impl<'v> Source<'v> {
    pub fn new(mvs: MetaValueStream<'v>) -> Self {
        Self(mvs)
    }
}

impl<'v> ValueProducer<'v> for Source<'v> {}

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

impl<'v> ValueProducer<'v> for Fixed<'v> {}

impl<'v> Iterator for Fixed<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Result::Ok)
    }
}

pub struct Raw<'v>(std::vec::IntoIter<Result<MetaVal<'v>, Error>>);

impl<'v> Raw<'v> {
    pub fn new(v: Vec<Result<MetaVal<'v>, Error>>) -> Self {
        Self(v.into_iter())
    }
}

impl<'v> ValueProducer<'v> for Raw<'v> {}

impl<'v> Iterator for Raw<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub struct Flatten<'v, I>(I, VecDeque<MetaVal<'v>>);

impl<'v, I> Flatten<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    pub fn new(it: I) -> Self {
        Self(it, VecDeque::new())
    }
}

impl<'v, I> ValueProducer<'v> for Flatten<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{}

impl<'v, I> Iterator for Flatten<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
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

pub struct Dedup<'v, I>(I, Option<MetaVal<'v>>);

impl<'v, I> Dedup<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    pub fn new(it: I) -> Self {
        Self(it, None)
    }
}

impl<'v, I> ValueProducer<'v> for Dedup<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{}

impl<'v, I> Iterator for Dedup<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
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

pub struct Unique<'v, I>(I, HashSet<MetaVal<'v>>);

impl<'v, I> Unique<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    pub fn new(it: I) -> Self {
        Self(it, HashSet::new())
    }
}

impl<'v, I> ValueProducer<'v> for Unique<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{}

impl<'v, I> Iterator for Unique<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
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

pub struct Filter<I>(I, UnaryPred);

impl<'v, I> Filter<I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    pub fn new(it: I, pred: UnaryPred) -> Self {
        Self(it, pred)
    }
}

impl<'v, I> ValueProducer<'v> for Filter<I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{}

impl<'v, I> Iterator for Filter<I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
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

pub struct Map<I>(I, UnaryConv);

impl<'v, I> Map<I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    pub fn new(it: I, conv: UnaryConv) -> Self {
        Self(it, conv)
    }
}

impl<'v, I> ValueProducer<'v> for Map<I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{}

impl<'v, I> Iterator for Map<I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next()? {
            Ok(mv) => Some(self.1(mv)),
            Err(err) => Some(Err(err)),
        }
    }
}

pub struct StepBy<I> {
    stream: I,
    curr: usize,
    n: usize,
}

impl<'v, I> StepBy<I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    // Can fail if step size is zero.
    pub fn new(it: I, n: usize) -> Result<Self, Error> {
        if n == 0 { Err(Error::ZeroStepSize) }
        else {
            Ok(Self {
                stream: it,
                curr: n,
                n,
            })
        }
    }
}

impl<'v, I> ValueProducer<'v> for StepBy<I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{}

impl<'v, I> Iterator for StepBy<I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.stream.next()? {
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

pub struct Chain<IA, IB>(IA, IB, bool);

impl<'v, IA, IB> Chain<IA, IB>
where
    IA: Iterator<Item = Result<MetaVal<'v>, Error>>,
    IB: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    pub fn new(it_a: IA, it_b: IB) -> Self {
        Self(it_a, it_b, false)
    }
}

impl<'v, IA, IB> ValueProducer<'v> for Chain<IA, IB>
where
    IA: Iterator<Item = Result<MetaVal<'v>, Error>>,
    IB: Iterator<Item = Result<MetaVal<'v>, Error>>,
{}

impl<'v, IA, IB> Iterator for Chain<IA, IB>
where
    IA: Iterator<Item = Result<MetaVal<'v>, Error>>,
    IB: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
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

pub struct Zip<IA, IB>(IA, IB);

impl<'v, IA, IB> Zip<IA, IB>
where
    IA: Iterator<Item = Result<MetaVal<'v>, Error>>,
    IB: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    pub fn new(it_a: IA, it_b: IB) -> Self {
        Self(it_a, it_b)
    }
}

impl<'v, IA, IB> ValueProducer<'v> for Zip<IA, IB>
where
    IA: Iterator<Item = Result<MetaVal<'v>, Error>>,
    IB: Iterator<Item = Result<MetaVal<'v>, Error>>,
{}

impl<'v, IA, IB> Iterator for Zip<IA, IB>
where
    IA: Iterator<Item = Result<MetaVal<'v>, Error>>,
    IB: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
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

pub struct Skip<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    it: I,
    curr: usize,
    n: usize,
}

impl<'v, I> Skip<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    pub fn new(it: I, n: usize) -> Self {
        Self {
            it: it,
            curr: 0,
            n,
        }
    }
}

impl<'v, I> ValueProducer<'v> for Skip<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{}

impl<'v, I> Iterator for Skip<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.curr < self.n {
            self.curr += 1;
            let res_mv = self.it.next()?;

            if let Err(e) = res_mv { return Some(Err(e)) }
        }

        self.it.next()
    }
}

pub struct Take<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    it: I,
    curr: usize,
    n: usize,
}

impl<'v, I> Take<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    pub fn new(it: I, n: usize) -> Self {
        Self {
            it: it,
            curr: 0,
            n,
        }
    }
}

impl<'v, I> ValueProducer<'v> for Take<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{}

impl<'v, I> Iterator for Take<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr < self.n {
            self.curr += 1;
            self.it.next()
        }
        else {
            None
        }
    }
}

pub struct SkipWhile<I>(I, UnaryPred, bool);

impl<'v, I> SkipWhile<I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    pub fn new(it: I, u_pred: UnaryPred) -> Self {
        Self(it, u_pred, true)
    }
}

impl<'v, I> ValueProducer<'v> for SkipWhile<I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{}

impl<'v, I> Iterator for SkipWhile<I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
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

pub struct TakeWhile<I>(I, UnaryPred, bool);

impl<'v, I> TakeWhile<I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    pub fn new(it: I, u_pred: UnaryPred) -> Self {
        Self(it, u_pred, true)
    }
}

impl<'v, I> ValueProducer<'v> for TakeWhile<I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{}

impl<'v, I> Iterator for TakeWhile<I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
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

pub struct Intersperse<'v, I>(I, MetaVal<'v>, bool);

impl<'v, I> Intersperse<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    pub fn new(it: I, mv: MetaVal<'v>) -> Self {
        Self(it, mv, false)
    }
}

impl<'v, I> ValueProducer<'v> for Intersperse<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{}

impl<'v, I> Iterator for Intersperse<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.2 = !self.2;

        if self.2 { self.0.next() }
        else { Some(Ok(self.1.clone())) }
    }
}

impl<'v, I> FusedIterator for Intersperse<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{}

pub struct Interleave<IA, IB>(IA, IB, bool);

impl<'v, IA, IB> Interleave<IA, IB>
where
    IA: Iterator<Item = Result<MetaVal<'v>, Error>>,
    IB: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    pub fn new(it_a: IA, it_b: IB) -> Self {
        Self(it_a, it_b, false)
    }
}

impl<'v, IA, IB> ValueProducer<'v> for Interleave<IA, IB>
where
    IA: Iterator<Item = Result<MetaVal<'v>, Error>>,
    IB: Iterator<Item = Result<MetaVal<'v>, Error>>,
{}

impl<'v, IA, IB> Iterator for Interleave<IA, IB>
where
    IA: Iterator<Item = Result<MetaVal<'v>, Error>>,
    IB: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.2 = !self.2;

        if self.2 { self.0.next() }
        else { self.1.next() }
    }
}

impl<'v, IA, IB> FusedIterator for Interleave<IA, IB>
where
    IA: Iterator<Item = Result<MetaVal<'v>, Error>>,
    IB: Iterator<Item = Result<MetaVal<'v>, Error>>,
{}
