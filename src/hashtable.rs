use crate::window::{Window, MAX_MATCH_LEN, WINDOW_SIZE};

/// How many hash-chains we manage.
const HASH_CHAIN_COUNT: usize = 4096;
/// How long a single hash-chain can grow.
const HASH_CHAIN_LENGTH: usize = 64;

/// The nil-value in the hash-chain (meaning the end of chain).
const HASH_NIL: u16 = u16::MAX;

/// A simple hash-table that manages chains of hash values.
pub struct HashTable {
    chain_count: usize,
    chain_length: usize,
    chain_offsets: Box<[usize]>,
    data: Box<[u16]>,
}

impl HashTable {
    pub fn new() -> Self {
        Self {
            // Hashing
            chain_count: HASH_CHAIN_COUNT,
            chain_length: HASH_CHAIN_LENGTH,
            chain_offsets: vec![0; HASH_CHAIN_COUNT].into_boxed_slice(),
            data: vec![HASH_NIL; HASH_CHAIN_COUNT * HASH_CHAIN_LENGTH].into_boxed_slice(),
        }
    }

    /// Hashes the given bytes. Returns the index of the corresponging
    /// hash-chain.
    pub fn hash(&self, key: &[u8]) -> usize {
        // FNV hash

        assert!(key.len() == 3);

        const FNV_BASIS: u64 = 0xcbf29ce484222325;
        const FNV_PRIME: u64 = 0x100000001b3;

        let mut result = FNV_BASIS;
        for b in key {
            result ^= *b as u64;
            result = result.wrapping_mul(FNV_PRIME);
        }
        result as usize % self.chain_count
    }

    /// Returns the hash chain for the given hash value.
    fn hash_chain_for_hash(&self, hash: usize) -> &[u16] {
        let offs = hash * self.chain_length;
        &self.data[offs..(offs + self.chain_length)]
    }

    /// Returns the hash chain for the given hash value.
    fn hash_chain_for_hash_mut(&mut self, hash: usize) -> &mut [u16] {
        let offs = hash * self.chain_length;
        &mut self.data[offs..(offs + self.chain_length)]
    }

    /// Returns the maximum match length of the key and the given offset of the
    /// sliding window.
    fn match_length(&self, key: &[u8], index: usize, window: &Window) -> usize {
        let max_offs = std::cmp::min(key.len(), MAX_MATCH_LEN);
        for i in 0..max_offs {
            let w_idx = (index + i) % WINDOW_SIZE;
            if w_idx == window.position || key[i] != window.data[w_idx] {
                return i;
            }
        }
        max_offs
    }

    /// Inserts the given value into the given hash-chain.
    pub fn insert(&mut self, chain_idx: usize, val: u16) {
        let offs = self.chain_offsets[chain_idx];
        self.hash_chain_for_hash_mut(chain_idx)[offs] = val;
        self.chain_offsets[chain_idx] = (offs + 1) % self.chain_length;
    }

    /// Searches for a backreference using the given chain index, input and
    /// sliding window. Returns the distance-length tuple, if a match is found.
    pub fn reference(
        &self,
        chain_idx: usize,
        upcoming: &[u8],
        window: &Window,
    ) -> Option<(usize, usize)> {
        let chain_len = self.chain_length;
        let mut chain_offset = self.chain_offsets[chain_idx];
        let mut longest_len = 0;
        let mut longest_idx = 0;
        let chain = self.hash_chain_for_hash(chain_idx);
        for _ in 0..chain_len {
            chain_offset = chain_offset.wrapping_sub(1) % chain_len;
            if chain[chain_offset] == HASH_NIL {
                break;
            }
            let idx = chain[chain_offset];
            let match_len = self.match_length(upcoming, idx as usize, window);
            if longest_len < match_len && match_len <= MAX_MATCH_LEN {
                longest_len = match_len;
                longest_idx = idx;
            }
        }
        if longest_len < 3 {
            None
        } else {
            let dist = window.distance_from(longest_idx as usize);
            Some((dist, longest_len))
        }
    }
}
