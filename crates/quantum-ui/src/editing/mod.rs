//! Renderer-independent single-line editing. All positions are UTF-8 byte offsets.
//! UI adapters translate platform UTF-16 ranges only at the input boundary.
mod bindings;
mod numeric;
mod unicode;
pub use bindings::*;
pub use numeric::*;
use std::{collections::VecDeque, ops::Range};
pub use unicode::{from_utf16, single_line, to_utf16};

/// Maximum UTF-8 bytes retained in a field.
pub const MAX_BYTES: usize = 8192;
/// Maximum undo transactions (redo has the same bound).
pub const HISTORY_LIMIT: usize = 100;
/// A movement or deletion boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    /// Extended grapheme cluster.
    Grapheme,
    /// Unicode word or punctuation run, skipping adjacent whitespace.
    Word,
    /// Beginning or end of this single line.
    Line,
}
/// Explicit cancellation policy, independent of the containing dialog.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EscapePolicy {
    /// Keep live edits and blur. The next Escape belongs to the container.
    #[default]
    Blur,
    /// Restore the last value set or committed, then blur.
    Revert,
}
/// Source of a replacement, controlling undo grouping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKind {
    /// Adjacent insertions within 750 ms form a typing run.
    Typing,
    /// One independent replacement, paste, deletion or scrub gesture.
    Atomic,
    /// One transaction across successive marked-text updates.
    Composition,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Snapshot {
    text: String,
    selection: Range<usize>,
    reversed: bool,
}
#[derive(Clone, Debug)]
struct Run {
    kind: EditKind,
    time: u64,
    head: usize,
}
/// A complete text editor without window, clipboard or renderer dependencies.
#[derive(Clone, Debug, Default)]
pub struct Editor {
    state: Snapshot,
    marked: Option<Range<usize>>,
    committed: String,
    undo: VecDeque<Snapshot>,
    redo: Vec<Snapshot>,
    run: Option<Run>,
    numeric: Option<NumericPolicy>,
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            text: String::new(),
            selection: 0..0,
            reversed: false,
        }
    }
}
impl Editor {
    /// Execute a text-only binding. Returns false for commands needing the UI
    /// (clipboard, focus, commit notification, Escape policy, or a popup).
    pub fn apply(&mut self, command: Command, time: u64) -> bool {
        match command {
            Command::Move(unit, forward, extend) => self.move_to(unit, forward, extend),
            Command::Delete(unit, forward) => self.delete(unit, forward, time),
            Command::Vertical(up, shift, fine) => {
                if self.numeric.is_some() {
                    self.step(if up { 1 } else { -1 }, shift, fine, time);
                } else {
                    self.move_to(Unit::Line, !up, shift);
                }
            }
            Command::Undo => self.undo(),
            Command::Redo => self.redo(),
            _ => return false,
        }
        true
    }
    /// Current text. Mutation goes through editing operations to preserve invariants.
    pub fn text(&self) -> &str {
        &self.state.text
    }
    /// Sorted selection range.
    pub fn selection(&self) -> Range<usize> {
        self.state.selection.clone()
    }
    /// Whether the active end precedes the anchor.
    pub fn reversed(&self) -> bool {
        self.state.reversed
    }
    /// Active end of the selection.
    pub fn head(&self) -> usize {
        if self.state.reversed {
            self.state.selection.start
        } else {
            self.state.selection.end
        }
    }
    /// Fixed end of the selection.
    pub fn anchor(&self) -> usize {
        if self.state.reversed {
            self.state.selection.end
        } else {
            self.state.selection.start
        }
    }
    /// Marked composition range, if any.
    pub fn marked(&self) -> Option<Range<usize>> {
        self.marked.clone()
    }
    /// Numeric validation and stepping policy.
    pub fn numeric(&self) -> Option<NumericPolicy> {
        self.numeric
    }
    /// Set numeric mode and establish a valid committed value.
    pub fn set_numeric(&mut self, policy: NumericPolicy, value: f64) {
        self.numeric = Some(policy);
        self.set(&policy.format(value));
    }
    /// Replace the model value, establishing a new baseline and empty history.
    pub fn set(&mut self, text: &str) {
        let mut text = single_line(text);
        let end = unicode::floor(&text, MAX_BYTES.min(text.len()));
        text.truncate(end);
        self.state = Snapshot {
            selection: text.len()..text.len(),
            text,
            reversed: false,
        };
        self.committed = self.state.text.clone();
        self.marked = None;
        self.undo.clear();
        self.redo.clear();
        self.run = None;
    }
    /// End the current undo coalescing run, e.g. on focus changes.
    pub fn break_run(&mut self) {
        self.run = None;
    }
    /// Place the selection at grapheme boundaries, preserving direction.
    pub fn select(&mut self, anchor: usize, head: usize) {
        let anchor = unicode::floor(self.text(), anchor.min(self.text().len()));
        let head = unicode::floor(self.text(), head.min(self.text().len()));
        self.state.selection = anchor.min(head)..anchor.max(head);
        self.state.reversed = head < anchor;
        self.break_run();
    }
    /// Select the entire line.
    pub fn select_all(&mut self) {
        self.select(0, self.text().len());
    }
    /// Select the Unicode word, emoji, punctuation run or whitespace under a click.
    pub fn word_at(&self, offset: usize) -> Range<usize> {
        unicode::word_at(self.text(), offset)
    }
    /// Extend a double-click drag by whole units, keeping its original word selected.
    pub fn select_words(&mut self, original: Range<usize>, head: usize) {
        let target = self.word_at(head);
        if head < original.start {
            self.select(original.end, target.start);
        } else {
            self.select(original.start, target.end.max(original.end));
        }
    }
    fn boundary(&self, unit: Unit, forward: bool) -> usize {
        match unit {
            Unit::Line => {
                if forward {
                    self.text().len()
                } else {
                    0
                }
            }
            Unit::Grapheme => unicode::adjacent(self.text(), self.head(), forward),
            Unit::Word => unicode::word_boundary(self.text(), self.head(), forward),
        }
    }
    /// Move or extend. Plain arrows collapse a selection without an extra step;
    /// modified arrows move from the active end to their requested boundary.
    pub fn move_to(&mut self, unit: Unit, forward: bool, extend: bool) {
        let next = if unit == Unit::Grapheme && !extend && !self.selection().is_empty() {
            if forward {
                self.selection().end
            } else {
                self.selection().start
            }
        } else {
            self.boundary(unit, forward)
        };
        self.select(if extend { self.anchor() } else { next }, next);
        self.marked = None;
    }
    fn record(&mut self, kind: EditKind, time: u64) {
        let coalesce = self.run.as_ref().is_some_and(|run| {
            run.kind == kind
                && match kind {
                    EditKind::Typing => {
                        time.saturating_sub(run.time) <= 750
                            && run.head == self.head()
                            && self.selection().is_empty()
                    }
                    EditKind::Composition => true,
                    EditKind::Atomic => false,
                }
        });
        if !coalesce {
            self.undo.push_back(self.state.clone());
            if self.undo.len() > HISTORY_LIMIT {
                self.undo.pop_front();
            }
        }
        self.redo.clear();
        self.run = Some(Run {
            kind,
            time,
            head: self.head(),
        });
    }
    /// Replace a byte range, current marked text, or selection. Returns whether
    /// text changed; invalid/oversized input cannot partially mutate the model.
    pub fn replace(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        kind: EditKind,
        time: u64,
    ) -> bool {
        self.replace_inner(range, text, kind, time)
            .is_some_and(|(_, changed)| changed)
    }
    fn replace_inner(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        kind: EditKind,
        time: u64,
    ) -> Option<(Range<usize>, bool)> {
        let range = range
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selection());
        // Native composition ranges may intentionally split a grapheme (dead
        // keys replace a combining mark). Only user navigation snaps graphemes.
        let start = unicode::scalar_floor(self.text(), range.start.min(range.end));
        let end = unicode::scalar_ceil(self.text(), range.end.max(range.start));
        let mut inserted = single_line(text);
        if let Some(policy) = self.numeric {
            inserted = policy.filter(&self.text()[..start], &inserted, &self.text()[end..]);
            // Reject invalid typing without accidentally deleting a selection.
            if !text.is_empty() && inserted.is_empty() {
                return None;
            }
        }
        if self.text().len() - (end - start) + inserted.len() > MAX_BYTES {
            return None;
        }
        let changed = self.text()[start..end] != inserted;
        if changed {
            if kind == EditKind::Typing
                && (start != end || start != self.head() || inserted.is_empty())
            {
                self.break_run();
            }
            self.record(kind, time);
        }
        self.state.text.replace_range(start..end, &inserted);
        let head = start + inserted.len();
        let caret = unicode::ceil(self.text(), head);
        self.state.selection = caret..caret;
        self.state.reversed = false;
        self.marked = None;
        if let Some(run) = &mut self.run {
            run.head = caret;
        }
        Some((start..head, changed))
    }
    /// Replace marked text as a single undo transaction. Selected offsets are
    /// UTF-16 offsets relative to the composition, as required by native IMEs.
    pub fn compose(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
        time: u64,
    ) {
        let Some((marked, _)) = self.replace_inner(range, text, EditKind::Composition, time) else {
            return;
        };
        let (start, end) = (marked.start, marked.end);
        self.marked = Some(start..end);
        if let Some(selected) = selected {
            let selection = from_utf16(&self.text()[start..end], selected);
            self.state.selection = start + selection.start..start + selection.end;
        }
    }
    /// Finish composition without losing its undo transaction.
    pub fn unmark(&mut self) {
        self.marked = None;
        let anchor = self.anchor();
        let head = self.head();
        self.select(anchor, head);
        self.break_run();
    }
    /// Delete the selection or the requested boundary range as one transaction.
    pub fn delete(&mut self, unit: Unit, forward: bool, time: u64) {
        let range = if self.selection().is_empty() {
            let head = self.head();
            let end = self.boundary(unit, forward);
            head.min(end)..head.max(end)
        } else {
            self.selection()
        };
        self.replace(Some(range), "", EditKind::Atomic, time);
        self.break_run();
    }
    /// Restore a transaction. Empty history is deliberately a no-op.
    pub fn undo(&mut self) {
        if let Some(previous) = self.undo.pop_back() {
            self.redo.push(std::mem::replace(&mut self.state, previous));
        }
        self.marked = None;
        self.break_run();
    }
    /// Reapply a transaction. Empty history is deliberately a no-op.
    pub fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo
                .push_back(std::mem::replace(&mut self.state, next));
        }
        self.marked = None;
        self.break_run();
    }
    /// Validate numeric input or restore its baseline, then establish a commit.
    pub fn commit(&mut self) {
        if let Some(policy) = self.numeric {
            self.state.text = policy
                .parse(self.text())
                .map(|v| policy.format(v))
                .unwrap_or_else(|| self.committed.clone());
            let head = self.head().min(self.text().len());
            self.select(head, head);
        }
        self.committed = self.state.text.clone();
        self.unmark();
    }
    /// Validate a domain-specific draft before establishing a committed value.
    /// Rejection preserves the draft, selection and previous Escape baseline.
    pub fn commit_if_valid(&mut self, validate: impl FnOnce(&str) -> bool) -> bool {
        if !validate(self.text()) {
            return false;
        }
        self.commit();
        true
    }
    /// Apply cancellation before the UI adapter blurs the field.
    pub fn escape(&mut self, policy: EscapePolicy) {
        if policy == EscapePolicy::Revert {
            let text = self.committed.clone();
            self.set(&text);
        }
        self.unmark();
    }
    /// Apply a numeric step; ordinary fields ignore this command.
    pub fn step(&mut self, direction: i8, coarse: bool, fine: bool, time: u64) {
        if let Some(policy) = self.numeric {
            let value = policy
                .parse(self.text())
                .or_else(|| policy.parse(&self.committed))
                .unwrap_or(policy.min);
            let text = policy.format(policy.step(value, direction, coarse, fine));
            self.replace(Some(0..self.text().len()), &text, EditKind::Atomic, time);
        }
    }
    /// Update one scrub gesture. Call `break_run` before and after the gesture.
    pub fn scrub(&mut self, initial: f64, delta: f64, coarse: bool, time: u64) {
        if let Some(policy) = self.numeric {
            let text = policy.format(policy.scrub(initial, delta, coarse));
            self.replace(
                Some(0..self.text().len()),
                &text,
                EditKind::Composition,
                time,
            );
        }
    }
}
#[cfg(test)]
mod tests;
