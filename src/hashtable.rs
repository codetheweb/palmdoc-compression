use crate::window::{Window, MAX_MATCH_LEN, WINDOW_SIZE};

/// How many hash-chains we manage.
const HASH_CHAIN_COUNT: usize = 4096;
/// How long a single hash-chain can grow.
const HASH_CHAIN_LENGTH: usize = 32;

/// The nil-value in the hash-chain (meaning the end of chain).
const HASH_NIL: u16 = u16::MAX;

// Note: since HASH_CHAIN_COUNT is a power of 2, we use bitwise AND for modulus.
// E.x. % HASH_CHAIN_COUNT -> & (HASH_CHAIN_COUNT - 1) to compute the modulus.

/// A simple hash-table that manages chains of hash values.
pub struct HashTable {
    chain_offsets: [usize; HASH_CHAIN_COUNT],
    data: [u16; HASH_CHAIN_COUNT * HASH_CHAIN_LENGTH],
}

impl HashTable {
    pub fn new() -> Self {
        Self {
            // Hashing
            chain_offsets: [0; HASH_CHAIN_COUNT],
            data: [HASH_NIL; HASH_CHAIN_COUNT * HASH_CHAIN_LENGTH],
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
        result as usize & (HASH_CHAIN_COUNT - 1)
    }

    /// Returns the hash chain for the given hash value.
    fn hash_chain_for_hash(&self, hash: usize) -> &[u16] {
        let offs = hash * HASH_CHAIN_LENGTH;
        &self.data[offs..(offs + HASH_CHAIN_LENGTH)]
    }

    /// Returns the hash chain for the given hash value.
    fn hash_chain_for_hash_mut(&mut self, hash: usize) -> &mut [u16] {
        let offs = hash * HASH_CHAIN_LENGTH;
        &mut self.data[offs..(offs + HASH_CHAIN_LENGTH)]
    }

    /// Returns the maximum match length of the key and the given offset of the
    /// sliding window.
    fn match_length(&self, key: &[u8], index: usize, window: &Window) -> usize {
        let max_offs = std::cmp::min(key.len(), MAX_MATCH_LEN);
        for (i, byte) in key.iter().enumerate().take(max_offs) {
            let w_idx = (index + i) % WINDOW_SIZE;
            if w_idx == window.position || *byte != window.data[w_idx] {
                return i;
            }
        }
        max_offs
    }

    /// Inserts the given value into the given hash-chain.
    pub fn insert(&mut self, chain_idx: usize, val: u16) {
        let offs = self.chain_offsets[chain_idx];
        self.hash_chain_for_hash_mut(chain_idx)[offs] = val;
        self.chain_offsets[chain_idx] = (offs + 1) & (HASH_CHAIN_LENGTH - 1);
    }

    /// Searches for a backreference using the given chain index, input and
    /// sliding window. Returns the distance-length tuple, if a match is found.
    pub fn reference(
        &self,
        chain_idx: usize,
        upcoming: &[u8],
        window: &Window,
        current_absolute_offset: usize,
    ) -> Option<(usize, usize)> {
        let mut chain_offset = self.chain_offsets[chain_idx];
        let mut longest_len = 0;
        let mut longest_idx = 0;
        let chain = self.hash_chain_for_hash(chain_idx);
        for _ in 0..HASH_CHAIN_LENGTH {
            chain_offset = chain_offset.wrapping_sub(1) & (HASH_CHAIN_LENGTH - 1);
            if chain[chain_offset] == HASH_NIL {
                break;
            }
            let idx = chain[chain_offset] as usize;
            let match_len = self.match_length(upcoming, idx, window);
            if longest_len < match_len
                && match_len <= MAX_MATCH_LEN
                && idx < current_absolute_offset
            {
                longest_len = match_len;
                longest_idx = idx;
            }
        }
        if longest_len < 3 {
            None
        } else {
            Some((current_absolute_offset - longest_idx, longest_len))
        }
    }
}
