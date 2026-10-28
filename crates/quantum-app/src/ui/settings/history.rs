//! Navigation history for Settings sections.
use super::Section;

/// Browser-style history: selecting a new section after going back drops the forward branch.
pub(super) struct SectionHistory {
    entries: Vec<Section>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_moves_back_and_forward_and_stops_at_boundaries() {
        let mut history = SectionHistory::new(Section::General);
        assert!(!history.can_back());
        assert!(!history.can_forward());
        history.back();
        history.forward();
        assert_eq!(history.current(), Section::General);
        history.visit(Section::Appearance);
        history.visit(Section::Viewport);
        history.back();
        assert_eq!(history.current(), Section::Appearance);
        assert!(history.can_back());
        assert!(history.can_forward());
        history.back();
        history.back();
        assert_eq!(history.current(), Section::General);
        history.forward();
        history.forward();
        history.forward();
        assert_eq!(history.current(), Section::Viewport);
        assert!(!history.can_forward());
    }

    #[test]
    fn history_preserves_forward_on_reselection_and_truncates_on_new_navigation() {
        let mut history = SectionHistory::new(Section::General);
        history.visit(Section::Appearance);
        history.visit(Section::Viewport);
        history.back();
        history.visit(Section::Appearance);
        assert!(history.can_forward());
        assert_eq!(history.entries.len(), 3);
        history.visit(Section::Agent);
        assert!(!history.can_forward());
        assert_eq!(
            history.entries,
            vec![Section::General, Section::Appearance, Section::Agent]
        );
        history.visit(Section::Agent);
        history.back();
        assert_eq!(history.current(), Section::Appearance);
    }
}
