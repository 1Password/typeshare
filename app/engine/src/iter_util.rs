//! Some extra iterator methods.

/// Iterator extension trait.
pub trait IterExt: Iterator {
    /// Fallible version of `any()`. Short circuits
    /// on true or error.
    #[inline]
    fn try_any<F, E>(&mut self, mut f: F) -> Result<bool, E>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Result<bool, E>,
    {
        for x in self.by_ref() {
            if f(x)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl<T> IterExt for std::slice::Iter<'_, T> {}
