//! Navigation history for Settings sections.
use super::Section;

/// Browser-style history: selecting a new section after going back drops the forward branch.
pub(super) struct SectionHistory {
    pub(super) entries: Vec<Section>,
    cursor: usize,
}
impl SectionHistory {
    pub(super) fn new(section: Section) -> Self {
        Self {
            entries: vec![section],
            cursor: 0,
        }
    }
    pub(super) fn current(&self) -> Section {
        self.entries[self.cursor]
    }
    pub(super) fn can_back(&self) -> bool {
        self.cursor > 0
    }
    pub(super) fn can_forward(&self) -> bool {
        self.cursor + 1 < self.entries.len()
    }
    pub(super) fn visit(&mut self, section: Section) {
        if self.current() != section {
            self.entries.truncate(self.cursor + 1);
            self.entries.push(section);
            self.cursor += 1;
        }
    }
    pub(super) fn back(&mut self) {
        if self.can_back() {
            self.cursor -= 1;
        }
    }
    pub(super) fn forward(&mut self) {
        if self.can_forward() {
            self.cursor += 1;
        }
    }
}
