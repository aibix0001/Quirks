pub struct BufferManager {
    buffers: Vec<crate::buffer::Buffer>,
    current: usize,
}

impl BufferManager {
    pub fn new() -> Self {
        Self {
            buffers: Vec::new(),
            current: 0,
        }
    }

    /// Open a new buffer from a file and push it to the manager
    pub fn open_file(&mut self, path: &str) -> anyhow::Result<()> {
        let buffer = crate::buffer::Buffer::from_file(path)?;
        self.buffers.push(buffer);
        self.current = self.buffers.len() - 1;
        Ok(())
    }

    /// Return the current buffer
    pub fn current_buffer(&mut self) -> &mut crate::buffer::Buffer {
        &mut self.buffers[self.current]
    }

    /// Switch to a buffer by index (0-based)
    pub fn switch_to(&mut self, idx: usize) -> anyhow::Result<()> {
        if idx >= self.buffers.len() {
            return Err(anyhow::anyhow!("Buffer index out of range"));
        }
        self.current = idx;
        Ok(())
    }

    /// Switch to next buffer (wrap around)
    pub fn next_buffer(&mut self) {
        if !self.buffers.is_empty() {
            self.current = (self.current + 1) % self.buffers.len();
        }
    }

    /// Switch to previous buffer (wrap around)
    pub fn prev_buffer(&mut self) {
        if !self.buffers.is_empty() {
            if self.current == 0 {
                self.current = self.buffers.len() - 1;
            } else {
                self.current -= 1;
            }
        }
    }

    /// Close current buffer and remove it
    pub fn close_current(&mut self) -> anyhow::Result<()> {
        if self.buffers.is_empty() {
            return Ok(());
        }
        self.buffers.remove(self.current);
        if self.current >= self.buffers.len() && !self.buffers.is_empty() {
            self.current = self.buffers.len() - 1;
        }
        Ok(())
    }

    /// Check if there are open buffers
    pub fn has_buffers(&self) -> bool {
        !self.buffers.is_empty()
    }

    /// Get count of open buffers
    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_manager_new() {
        let bm = BufferManager::new();
        assert!(!bm.has_buffers());
        assert_eq!(bm.buffer_count(), 0);
    }

    #[test]
    fn test_next_prev_empty() {
        let mut bm = BufferManager::new();
        // Should not panic on empty
        bm.next_buffer();
        bm.prev_buffer();
        assert!(!bm.has_buffers());
    }

    #[test]
    fn test_switch_to_invalid() {
        let mut bm = BufferManager::new();
        assert!(bm.switch_to(0).is_err());
        assert!(bm.switch_to(5).is_err());
    }
}
