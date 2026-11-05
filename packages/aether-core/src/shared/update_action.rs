/// Outcome of a read-modify-write closure: whether the modified value has to be stored.
pub enum UpdateAction<R> {
    Save(R),
    NoChanges(R),
}

impl UpdateAction<()> {
    /// `Save` when the closure changed something, `NoChanges` otherwise.
    pub fn from_changed(changed: bool) -> Self {
        if changed {
            Self::Save(())
        } else {
            Self::NoChanges(())
        }
    }
}
