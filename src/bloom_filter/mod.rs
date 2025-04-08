#[cfg(test)]
mod tests;

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    marker::PhantomData,
};

/// Probabilistic structure that allows to identify if we have probably meet some element of a set
/// before.
///
/// If the filter negatively responds to contains we can be 100% sure that the value was not added
/// to filter. You'll get expected level of false positives only if you utilize all requested
/// capacity, otherwise you'll may have even smaller values.
///
/// You must configure planned capacity of elements and desired number of false-positives.
/// Both values determine the size of underlying data structure. If we need to put more values
/// to the filter or receive false-positive less probable, then the underlying structure will be
/// bigger.
///
/// The real underlying data structure size depends on planned capacity, but may be less or even
/// bigger than planned capacity.
#[derive(Debug)]
pub struct BloomFilter<T> {
    filter: Vec<u8>,
    hash_functions: usize,
    _phantom: PhantomData<T>,
}

impl<T> BloomFilter<T> {
    /// Creates a new filter with new configured capacity and false positives' probability.
    pub fn new(planned_capacity: usize, false_positives_probability: f64) -> Self {
        let planned_capacity = planned_capacity as f64;

        let bits =
            (-1.0 * planned_capacity * false_positives_probability.ln()) / 2_f64.ln().powf(2.0);

        let bytes = (bits / 8.0).ceil() as usize;

        let hash_functions = (bits / planned_capacity * 2_f64.ln()).ceil() as usize;

        Self {
            filter: vec![0; bytes],
            hash_functions,
            _phantom: Default::default(),
        }
    }
}

impl<T> BloomFilter<T>
where
    T: Hash,
{
    pub fn add(&mut self, item: T) {
        let bit_indexes =
            Self::get_byte_index_and_mask_pairs(self.hash_functions, self.filter.len(), &item);

        for (byte_index, bit_mask) in bit_indexes {
            self.filter[byte_index] |= bit_mask;
        }
    }

    pub fn contains(&self, item: &T) -> bool {
        let bit_indexes =
            Self::get_byte_index_and_mask_pairs(self.hash_functions, self.filter.len(), item);

        for (byte_index, bit_mask) in bit_indexes {
            if self.filter[byte_index] & bit_mask == 0 {
                return false;
            }
        }

        true
    }

    fn get_byte_index_and_mask_pairs(
        functions: usize,
        size: usize,
        item: &T,
    ) -> impl Iterator<Item = (usize, u8)> {
        let bit_size = size * 8;

        (0..functions).map(move |i| {
            let mut hasher = DefaultHasher::new();

            item.hash(&mut hasher);
            i.hash(&mut hasher);

            Self::bit_index_to_byte_index_and_mask(hasher.finish() as usize % bit_size)
        })
    }

    fn bit_index_to_byte_index_and_mask(bit_index: usize) -> (usize, u8) {
        let bit_in_byte = (bit_index % 8) as u8;
        let bit_n = (1 << bit_in_byte) as u8;

        (bit_index / 8, bit_n)
    }
}
