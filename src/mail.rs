//! Traits to share data in snui.

/// Mail refers to the idea of a mailing service.
///
/// In the scope of snui, a Mail is an interface widgets can use exchange messages with the application.
/// This allows widgets to operate idependently from the application and keep most of the complexity inside the trait implementation.
///
/// # Usage
///
/// ```
/// ```
pub trait Mail<'a, M, D, U> {
    fn get(&'a self, message: M) -> Option<U>;
    fn send(&'a mut self, message: M, data: D) -> Option<U>;
}

impl<'a, F, M, D, U> Mail<'a, M, D, U> for F
where
    F: Fn(M, Option<D>) -> Option<U>,
    F: FnMut(M, Option<D>) -> Option<U>,
{
    fn get(&'a self, message: M) -> Option<U> {
        (self)(message, None)
    }
    fn send(&'a mut self, message: M, data: D) -> Option<U> {
        (self)(message, Some(data))
    }
}
