/// Helper trait for method chaining
pub trait Pipe {
    #[must_use]
    fn pipe<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
        Self: Sized;
}

impl<T> Pipe for T {
    fn pipe<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        f(self)
    }
}
