#[cfg(test)]
mod tests;

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    marker::PhantomData,
    ops::{Add, Sub},
};

const DEFAULT_FALSE_POSITIVE_PROBABILITY: f64 = 0.01;

#[derive(Debug)]
pub struct CountingBloomFilter<T, K: Default + Copy = usize> {
    filter: Vec<K>,
    hash_functions: usize,
    _phantom: PhantomData<T>,
}

impl<T> CountingBloomFilter<T> {
    pub fn with_planned_capacity(planned_capacity: usize) -> Self {
        Self::with_planned_capacity_and_false_positives_probability(
            planned_capacity,
            DEFAULT_FALSE_POSITIVE_PROBABILITY,
        )
    }

    pub fn with_planned_capacity_and_false_positives_probability(
        planned_capacity: usize,
        false_positives_probability: f64,
    ) -> Self {
        let planned_capacity = planned_capacity as f64;

        let counters = ((-1.0 * planned_capacity * false_positives_probability.ln())
            / 2_f64.ln().powf(2.0)) as usize;

        let hash_functions = (counters as f64 / planned_capacity * 2_f64.ln()).ceil() as usize;

        Self {
            filter: vec![Default::default(); counters],
            hash_functions,
            _phantom: Default::default(),
        }
    }
}

impl<T, K> CountingBloomFilter<T, K>
where
    T: Hash,
    K: Default + From<u8> + Add<Output = K> + Sub<Output = K> + Eq + Copy,
{
    pub fn add(&mut self, item: &T) {
        let indexes = Self::get_indexes(self.hash_functions, self.filter.len(), item);

        for index in indexes {
            self.filter[index] = self.filter[index] + 1.into();
        }
    }

    pub fn contains(&self, item: &T) -> bool {
        let indexes = Self::get_indexes(self.hash_functions, self.filter.len(), item);

        for index in indexes {
            if self.filter[index] == Default::default() {
                return false;
            }
        }

        true
    }

    pub fn remove(&mut self, item: &T) -> bool {
        let indexes = Self::get_indexes(self.hash_functions, self.filter.len(), item);

        for index in indexes.clone() {
            if self.filter[index] == Default::default() {
                return false;
            }
        }

        for index in indexes {
            self.filter[index] = self.filter[index] - 1.into();
        }

        true
    }

    fn get_indexes(functions: usize, size: usize, item: &T) -> impl Iterator<Item = usize> + Clone {
        (0..functions).map(move |i| {
            let mut hasher = DefaultHasher::new();

            item.hash(&mut hasher);
            i.hash(&mut hasher);

            hasher.finish() as usize % size
        })
    }
}
