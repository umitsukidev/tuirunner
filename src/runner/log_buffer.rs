#[derive(Debug, Clone, Default)]
pub struct LogBuffer {
    pub lines: Vec<String>,
    pub version: usize,
}

impl LogBuffer {
    pub fn push(&mut self, line: String) {
        self.lines.push(line);
        self.version = self.version.wrapping_add(1);
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.version = self.version.wrapping_add(1);
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }
}

#[cfg(test)]
mod tests {
    use super::LogBuffer;

    #[test]
    fn test_push_and_clear_update_contents_and_version() {
        let mut buffer = LogBuffer::default();

        buffer.push("first".to_string());
        buffer.push("second".to_string());
        assert_eq!(buffer.lines, ["first", "second"]);
        assert_eq!(buffer.version, 2);
        assert_eq!(buffer.len(), 2);

        buffer.clear();
        assert!(buffer.lines.is_empty());
        assert_eq!(buffer.version, 3);
    }
}
