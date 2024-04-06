/// The maximum allowed backreference distance which is also the size of the
/// sliding window.
pub const WINDOW_SIZE: usize = 2048;
/// The longest match length we allow.
pub const MAX_MATCH_LEN: usize = 10;

/// A very simple sliding window implementation.
pub struct Window {
    pub data: [u8; WINDOW_SIZE],
    pub position: usize,
}

impl Window {
    pub fn new() -> Self {
        Self {
            data: [0; WINDOW_SIZE],
            position: 0,
        }
    }

    /// Pushes a single byte into the sliding window.
    pub fn push(&mut self, byte: u8) {
        self.data[self.position] = byte;
        self.position = (self.position + 1) % WINDOW_SIZE;
    }

    pub fn advance(&mut self, len: usize) {
        self.position = (self.position + len) % WINDOW_SIZE;
    }

    /// Calculates the distance of the
    pub fn distance_from(&self, index: usize) -> usize {
        if self.position > index {
            self.position - index
        } else {
            WINDOW_SIZE - index + self.position
        }
    }

    /// Inserts a back-reference based on distance and backreference length.
    /// Returns the inserted region.
    pub fn push_reference(&mut self, dist: usize, len: usize) -> (&[u8], &[u8]) {
        assert!(dist <= WINDOW_SIZE);
        assert!(len <= MAX_MATCH_LEN);

        let wnd_start = (self.position + WINDOW_SIZE - dist) % WINDOW_SIZE;
        let mut wnd_idx = wnd_start;
        let ins_start = self.position;
        for _ in 0..len {
            self.data[self.position] = self.data[wnd_idx];
            self.position = (self.position + 1) % WINDOW_SIZE;
            wnd_idx = (wnd_idx + 1) % WINDOW_SIZE;
        }
        if ins_start <= self.position {
            // One slice
            (&self.data[ins_start..self.position], &self.data[0..0])
        } else {
            // Two slices
            (&self.data[ins_start..], &self.data[..self.position])
        }
    }
}
