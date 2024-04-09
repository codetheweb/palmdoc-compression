/// The maximum allowed backreference distance is the size of the window.
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
}
