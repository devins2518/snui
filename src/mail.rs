//! Traits to share data in snui.

/// Mail refers to the idea of a mailing service.
///
/// In the scope of snui, a `Mail` is an interface widgets can use exchange messages and data with yout application.
/// This allows widgets to operate independently from it and keep most of the complexity inside the trait implementation.
///
/// # Usage
///
/// ```
/// ```

pub trait Mail<'a, M, D, U> {
    fn get(&'a self, message: M) -> Option<U>;
    fn send(&'a mut self, message: M, data: D) -> Option<U>;
}
