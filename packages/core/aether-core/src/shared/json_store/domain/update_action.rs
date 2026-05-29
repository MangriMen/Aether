pub enum UpdateAction<R> {
    Save(R),
    NoChanges(R),
}
