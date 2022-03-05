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
use std::borrow::Borrow;

pub trait Mail<'a, M, D, U> {
    fn get(&'a self, message: M) -> Option<U>;
    fn send(&'a mut self, message: M, data: D) -> Option<U>;
}

pub trait BorrowMail<'a, M, D, U> {
    fn get<B: Borrow<M>>(&'a self, message: M) -> Option<U>;
    fn send<B: Borrow<M>>(&'a mut self, message: M, data: D) -> Option<U>;
}

impl<'a, M, D, U, T> BorrowMail<'a, M, D, U> for T
where
    T: for<'b> Mail<'a, &'b M, D, U>,
{
    fn get<B: Borrow<M>>(&'a self, message: M) -> Option<U> {
        self.get(message.borrow())
    }
    fn send<B: Borrow<M>>(&'a mut self, message: M, data: D) -> Option<U> {
        self.send(message.borrow(), data)
    }
}
