#[cfg(test)]
mod tests;

use crate::sstable::SsTable;
use std::{
    collections::BTreeMap,
    error::Error,
    fs::File,
    hash::Hash,
    io::{BufReader, BufWriter, Write},
};

pub struct LsmTree<K, V>
where
    K: Hash + Clone + Ord + bincode::Encode + bincode::Decode<()>,
    V: Clone + bincode::Encode + bincode::Decode<()>,
{
    map: BTreeMap<K, V>,
    memtable_size: usize,
    data_directory: String,
    ss_table_block_size: usize,
    level_0_ss_tables: usize,
}

#[derive(bincode::Encode, bincode::Decode)]
struct State {
    ss_table_block_size: usize,
    memtable_size: usize,
    level_0_ss_tables: usize,
}

impl<K, V> Drop for LsmTree<K, V>
where
    K: Hash + Clone + Ord + bincode::Encode + bincode::Decode<()>,
    V: Clone + bincode::Encode + bincode::Decode<()>,
{
    fn drop(&mut self) {
        self.flush().unwrap();
    }
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
        std::fs::create_dir_all(format!("{data_directory}/level0"))?;

        Ok(Self {
            map: BTreeMap::new(),
            memtable_size,
            data_directory,
            ss_table_block_size,
            level_0_ss_tables: 0,
        })
    }

    pub fn load(data_directory: String) -> Result<Self, Box<dyn Error>> {
        let reader = BufReader::new(File::open(format!("{data_directory}/state"))?);

        let State {
            ss_table_block_size,
            memtable_size,
            level_0_ss_tables,
        }: State = bincode::decode_from_reader(reader, bincode::config::standard())?;

        Ok(Self {
            map: BTreeMap::new(),
            memtable_size,
            data_directory,
            ss_table_block_size,
            level_0_ss_tables,
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

        for ss_table in (0..self.level_0_ss_tables).rev() {
            let path = format!("{}/level0/{ss_table}", self.data_directory);
            let ss_table = SsTable::<K, V>::load(path)?;

            if let Some(value) = ss_table.get(key)? {
                return Ok(Some(value.clone()));
            }
        }

        Ok(None)
    }

    pub fn flush(&mut self) -> Result<(), Box<dyn Error>> {
        let mut map = BTreeMap::new();
        std::mem::swap(&mut self.map, &mut map);

        let path = format!("{}/level0/{}", self.data_directory, self.level_0_ss_tables);
        let _ = SsTable::new(map, &path, self.ss_table_block_size)?;

        self.level_0_ss_tables += 1;

        let mut writer = BufWriter::new(File::create(format!("{}/state", self.data_directory))?);

        let state = State {
            ss_table_block_size: self.ss_table_block_size,
            memtable_size: self.memtable_size,
            level_0_ss_tables: self.level_0_ss_tables,
        };

        let encoded_state = bincode::encode_to_vec(state, bincode::config::standard())?;

        writer.write_all(encoded_state.as_slice())?;

        Ok(())
    }
}
