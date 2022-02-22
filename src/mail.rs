//! Traits to share data in snui.

/// Mail refers to the idea of a mailing service.
///
/// In the scope of snui, a Mail is an interface widgets can use exchange messages with the application.
/// This allows widgets to operate idependently from the application and keep most of the complexity inside the trait implementation.
///
/// # Usage
///
/// ```
///	/// This example is inspired by the slider
/// ```
pub trait Mail<M, D, U> {
    fn get(&self, _message: M) -> Option<U> {
        None
    }
    fn send(&mut self, _message: M, _data: D) -> Option<U> {
        None
    }
}

trait SimpleMail<M, D, U>: Mail<M, D, U> {
    fn send(&mut self, message: M) -> Option<U>;
}

impl<M, U, T> SimpleMail<M, (), U> for T
where
    T: Mail<M, (), U>,
{
    fn send(&mut self, message: M) -> Option<U> {
        Mail::send(self, message, ())
    }
}

impl<F, M, D, U> Mail<M, D, U> for F
where
    F: Fn(M, Option<D>) -> Option<U>,
    F: FnMut(M, Option<D>) -> Option<U>,
{
    fn get(&self, message: M) -> Option<U> {
        (self)(message, None)
    }
    fn send(&mut self, message: M, data: D) -> Option<U> {
        (self)(message, Some(data))
    }
}

/// Keeps track of the state of your application.
/// When your application needs to be updated, your widgets' sync method will be invoked.
///
/// If sync returns true, your widgets will receive a Sync event along with your Data.
pub trait Data {
    fn sync(&mut self) -> bool;
}
