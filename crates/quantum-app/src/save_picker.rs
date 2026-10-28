//! Cancellation gate for asynchronous native save-path requests.

/// Identifies the one picker whose result may still start a write.
#[derive(Default)]
pub struct SavePicker {
    generation: u64,
    active: Option<u64>,
}

impl SavePicker {
    /// Start a request, invalidating any previous request.
    pub fn begin(&mut self) -> u64 {
        self.generation += 1;
        self.active = Some(self.generation);
        self.generation
    }

    /// Whether the editor is waiting for a save-path response.
    pub fn waiting(&self) -> bool {
        self.active.is_some()
    }

    /// Abandon the request. Late native responses must be ignored.
    pub fn cancel(&mut self) {
        self.active = None;
    }

    /// Consume a response only if it belongs to the current request.
    pub fn complete(&mut self, request: u64) -> bool {
        if self.active != Some(request) {
            return false;
        }
        self.active = None;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::SavePicker;

    #[test]
    fn cancel_rejects_late_response_even_after_another_save_starts() {
        let mut picker = SavePicker::default();
        let old = picker.begin();
        picker.cancel();
        assert!(!picker.waiting());
        assert!(!picker.complete(old));
        let current = picker.begin();
        assert!(!picker.complete(old));
        assert!(picker.waiting());
        assert!(picker.complete(current));
        assert!(!picker.waiting());
        assert!(!picker.complete(current));
    }

    #[test]
    fn empty_or_failed_response_is_consumed_without_restarting() {
        let mut picker = SavePicker::default();
        let request = picker.begin();
        assert!(picker.complete(request));
        assert!(!picker.waiting());
        assert!(!picker.complete(request));
    }
}
