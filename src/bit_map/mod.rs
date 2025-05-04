#[cfg(test)]
mod tests;

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub struct BitMap {
    map: Vec<u8>,
    bit_size: usize,
}

impl BitMap {
    pub fn new(size: usize) -> Self {
        let (mut bytes, rem) = (size / 8, size % 8);
        if rem > 0 {
            bytes += 1;
        }

        Self {
            map: vec![0; bytes],
            bit_size: size,
        }
    }

    pub fn is_set(&self, idx: usize) -> bool {
        let (byte_idx, mask) = self.get_byte_index_and_mask(idx);
        let byte = self.map[byte_idx];

        byte & mask != 0
    }

    pub fn set(&mut self, idx: usize) {
        let (byte_idx, mask) = self.get_byte_index_and_mask(idx);
        let byte = self.map[byte_idx];

        let new_byte = mask | byte;

        self.map[byte_idx] = new_byte;
    }

    pub fn reset(&mut self, idx: usize) {
        let (byte_idx, mask) = self.get_byte_index_and_mask(idx);
        let byte = self.map[byte_idx];

        let new_byte = byte & !mask;

        self.map[byte_idx] = new_byte;
    }

    pub fn bit_size(&self) -> usize {
        self.bit_size
    }

    pub fn byte_size(&self) -> usize {
        self.map.len()
    }

    fn get_byte_index_and_mask(&self, idx: usize) -> (usize, u8) {
        let bit_in_byte = (idx % 8) as u8;
        let mask = (1 << bit_in_byte) as u8;

        (idx / 8, mask)
    }
}
