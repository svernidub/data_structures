#[cfg(test)]
mod tests;

use crate::sstable::SsTable;
use std::{collections::BTreeMap, error::Error, hash::Hash};

pub struct LsmTree<K, V> {
    map: BTreeMap<K, V>,
    memtable_size: usize,
    data_directory: String,
    ss_table_block_size: usize,
    level_1_ss_tables: Vec<SsTable<K, V>>,
}

impl<K, V> LsmTree<K, V>
where
    K: Hash + Clone + Ord + bincode::Encode + bincode::Decode<()>,
    V: Clone + bincode::Encode + bincode::Decode<()>,
{
    pub fn new(
        data_directory: String,
        memtable_size: usize,
        ss_table_block_size: usize,
    ) -> Result<Self, Box<dyn Error>> {
        std::fs::create_dir_all(format!("{data_directory}/level1"))?;

        Ok(Self {
            map: BTreeMap::new(),
            memtable_size,
            data_directory,
            ss_table_block_size,
            level_1_ss_tables: Vec::new(),
        })
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<(), Box<dyn Error>> {
        if self.map.len() == self.memtable_size {
            self.flush()?;
        }

        self.map.insert(key, value);
        Ok(())
    }

    pub fn get(&self, key: &K) -> Result<Option<V>, Box<dyn Error>> {
        if let Some(value) = self.map.get(key) {
            return Ok(Some(value.clone()));
        };

        for ss_table in self.level_1_ss_tables.iter().rev() {
            if let Some(value) = ss_table.get(key)? {
                return Ok(Some(value.clone()));
            }
        }

        Ok(None)
    }

    pub fn flush(&mut self) -> Result<(), Box<dyn Error>> {
        let mut map = BTreeMap::new();
        std::mem::swap(&mut self.map, &mut map);

        let path = format!(
            "{}/level1/{}",
            self.data_directory,
            self.level_1_ss_tables.len()
        );

        SsTable::create_from_data(map, &path, self.ss_table_block_size)?;

        // TODO: table initialization should be enough, stop reload index and filters.
        self.level_1_ss_tables.push(SsTable::load(path)?);

        Ok(())
    }
}
