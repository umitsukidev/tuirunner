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
