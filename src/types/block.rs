use std::iter::FusedIterator;
use std::collections::BTreeMap as InnerMap;
use std::collections::btree_map::{IntoIter as InnerIntoIter, Iter as InnerIter};
use std::iter::FromIterator;

use crate::types::Value;

/// Represents a chunk of metadata for one item.
#[derive(Debug, Clone, Default)]
pub struct Block(InnerMap<String, Value>);

impl Block {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    pub fn get_full(&self, key: &str) -> Option<(&String, &Value)> {
        self.0.get_key_value(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.0.remove(key)
    }

    pub fn remove_full(&mut self, key: &str) -> Option<(String, Value)> {
        self.0.remove_entry(key)
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter(self.0.iter())
    }
}

impl FromIterator<(String, Value)> for Block {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (String, Value)>,
    {
        Self(iter.into_iter().collect())
    }
}

impl IntoIterator for Block {
    type Item = (String, Value);
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}

pub struct Iter<'a>(InnerIter<'a, String, Value>);

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a String, &'a Value);

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

pub struct IntoIter(InnerIntoIter<String, Value>);

impl Iterator for IntoIter {
    type Item = (String, Value);

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
