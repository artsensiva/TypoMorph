//! Metadata-only context validation. This is not an atomic edit transaction.
#[derive(Clone, PartialEq, Eq)]
pub struct FieldSnapshot {
    pub application: String,
    pub window: String,
    pub field: String,
    pub caret: i32,
    pub characters: i32,
}

#[derive(Default)]
pub struct ContextGuard {
    latest: Option<FieldSnapshot>,
}

impl ContextGuard {
    pub fn new(initial: Option<FieldSnapshot>) -> Self {
        Self { latest: initial }
    }
    pub fn clear(&mut self) {
        self.latest = None;
    }

    /// Only a single observed insertion in the same field may extend a word.
    /// Uncertain ordering resets the word rather than guessing what was typed.
    pub fn observe_insertion(&mut self, snapshot: Option<FieldSnapshot>) -> bool {
        let continuous = match (&self.latest, &snapshot) {
            (Some(old), Some(new)) => {
                old.application == new.application
                    && old.window == new.window
                    && old.field == new.field
                    && old.caret.checked_add(1) == Some(new.caret)
                    && old.characters.checked_add(1) == Some(new.characters)
            }
            _ => false,
        };
        self.latest = snapshot;
        continuous
    }

    pub fn unchanged(&self, current: Option<&FieldSnapshot>) -> bool {
        matches!((&self.latest, current), (Some(old), Some(new)) if old == new)
    }

    pub fn reset(&mut self, current: Option<FieldSnapshot>) {
        self.latest = current;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn field(caret: i32, characters: i32) -> FieldSnapshot {
        FieldSnapshot {
            application: ":1.10".into(),
            window: "/window".into(),
            field: "/entry".into(),
            caret,
            characters,
        }
    }
    #[test]
    fn tracks_insertions_and_requires_exact_pre_edit_state() {
        let mut guard = ContextGuard::new(Some(field(10, 20)));
        assert!(guard.observe_insertion(Some(field(11, 21))));
        assert!(guard.unchanged(Some(&field(11, 21))));
        assert!(!guard.unchanged(Some(&field(10, 21))));
        assert!(!guard.unchanged(Some(&field(11, 22))));
    }
    #[test]
    fn app_window_and_field_changes_invalidate_the_word() {
        for part in 0..3 {
            let mut next = field(11, 21);
            match part {
                0 => next.application.push('x'),
                1 => next.window.push('x'),
                _ => next.field.push('x'),
            }
            let mut guard = ContextGuard::new(Some(field(10, 20)));
            assert!(!guard.observe_insertion(Some(next)));
        }
    }
    #[test]
    fn cursor_movement_paste_deletion_and_unavailable_metadata_reset() {
        for next in [
            Some(field(9, 21)),
            Some(field(12, 22)),
            Some(field(9, 19)),
            None,
        ] {
            let mut guard = ContextGuard::new(Some(field(10, 20)));
            assert!(!guard.observe_insertion(next));
        }
    }
    #[test]
    fn lost_context_never_authorizes_a_replacement() {
        let mut guard = ContextGuard::new(Some(field(10, 20)));
        guard.clear();
        assert!(!guard.unchanged(None));
        assert!(!guard.unchanged(Some(&field(10, 20))));
        assert!(!guard.observe_insertion(Some(field(11, 21))));
        assert!(guard.observe_insertion(Some(field(12, 22))));
    }
}
