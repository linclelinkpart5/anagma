use std::iter::FusedIterator;
use std::vec::IntoIter as InnerIntoIter;
use std::slice::{Iter as InnerIter, IterMut as InnerIterMut};
use std::iter::{Extend, FromIterator};

use crate::types::Block;

/// Represents multiple chunks of metadata for an ordered collection of items.
#[derive(Debug, Clone, Default)]
pub struct BlockSeq(Vec<Block>);

impl BlockSeq {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn push(&mut self, block: Block) {
        self.0.push(block)
    }

    pub fn pop(&mut self) -> Option<Block> {
        self.0.pop()
    }

    pub fn insert(&mut self, idx: usize, block: Block) {
        self.0.insert(idx, block)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&Block> {
        self.0.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Block> {
        self.0.get_mut(index)
    }

    pub fn remove(&mut self, index: usize) -> Option<Block> {
        if index < self.0.len() {
            Some(self.0.remove(index))
        } else {
            None
        }
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter(self.0.iter())
    }

    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut(self.0.iter_mut())
    }
}

impl Extend<Block> for BlockSeq {
    fn extend<I: IntoIterator<Item = Block>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}

impl FromIterator<Block> for BlockSeq {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Block>,
    {
        Self(iter.into_iter().collect())
    }
}

impl IntoIterator for BlockSeq {
    type Item = Block;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}

pub struct Iter<'a>(InnerIter<'a, Block>);

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Block;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> FusedIterator for Iter<'a> {}

pub struct IterMut<'a>(InnerIterMut<'a, Block>);

impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut Block;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a> DoubleEndedIterator for IterMut<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<'a> ExactSizeIterator for IterMut<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> FusedIterator for IterMut<'a> {}

pub struct IntoIter(InnerIntoIter<Block>);

impl Iterator for IntoIter {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl DoubleEndedIterator for IntoIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl ExactSizeIterator for IntoIter {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl FusedIterator for IntoIter {}

