use std::iter::FusedIterator;
use std::iter::{Extend, FromIterator};

use indexmap::IndexMap as InnerMap;
use indexmap::map::{
    IntoIter as InnerIntoIter,
    Iter as InnerIter,
    IterMut as InnerIterMut,
    Keys as InnerKeys,
    Values as InnerValues,
    ValuesMut as InnerValuesMut,
};
use serde::{Serialize, Deserialize};

use crate::types::Block;

/// Represents multiple chunks of metadata for a mapping of items keyed by name.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(transparent)]
pub struct BlockMap(pub(crate) InnerMap<String, Block>);

impl BlockMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn insert(&mut self, key: String, value: Block) -> Option<Block> {
        self.0.insert(key, value)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, key: &str) -> Option<&Block> {
        self.0.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Block> {
        self.0.get_mut(key)
    }

    pub fn get_full(&self, key: &str) -> Option<(&String, &Block)> {
        self.0.get_key_value(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<Block> {
        self.0.remove(key)
    }

    pub fn remove_full(&mut self, key: &str) -> Option<(String, Block)> {
        self.0.remove_entry(key)
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter(self.0.iter())
    }

    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut(self.0.iter_mut())
    }

    pub fn keys(&self) -> Keys<'_> {
        Keys(self.0.keys())
    }

    pub fn values(&self) -> Values<'_> {
        Values(self.0.values())
    }

    pub fn values_mut(&mut self) -> ValuesMut<'_> {
        ValuesMut(self.0.values_mut())
    }

    // NOTE: Private method to help support in-crate usage.
    //       Kept private because efficient popping is not guranteed on all map
    //       types, and it would be better to hide that API.
    pub(crate) fn pop(&mut self) -> Option<(String, Block)> {
        self.0.pop()
    }
}

impl Extend<(String, Block)> for BlockMap {
    fn extend<I: IntoIterator<Item = (String, Block)>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}

impl FromIterator<(String, Block)> for BlockMap {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (String, Block)>,
    {
        Self(iter.into_iter().collect())
    }
}

impl IntoIterator for BlockMap {
    type Item = (String, Block);
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}

pub struct Iter<'a>(InnerIter<'a, String, Block>);

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a String, &'a Block);

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

pub struct IterMut<'a>(InnerIterMut<'a, String, Block>);

impl<'a> Iterator for IterMut<'a> {
    type Item = (&'a String, &'a mut Block);

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

pub struct Keys<'a>(InnerKeys<'a, String, Block>);

impl<'a> Iterator for Keys<'a> {
    type Item = &'a String;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a> DoubleEndedIterator for Keys<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<'a> ExactSizeIterator for Keys<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> FusedIterator for Keys<'a> {}

pub struct Values<'a>(InnerValues<'a, String, Block>);

impl<'a> Iterator for Values<'a> {
    type Item = &'a Block;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a> DoubleEndedIterator for Values<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<'a> ExactSizeIterator for Values<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> FusedIterator for Values<'a> {}

pub struct ValuesMut<'a>(InnerValuesMut<'a, String, Block>);

impl<'a> Iterator for ValuesMut<'a> {
    type Item = &'a mut Block;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a> DoubleEndedIterator for ValuesMut<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<'a> ExactSizeIterator for ValuesMut<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> FusedIterator for ValuesMut<'a> {}

pub struct IntoIter(InnerIntoIter<String, Block>);

impl Iterator for IntoIter {
    type Item = (String, Block);

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
