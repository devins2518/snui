//! Traits to share data in snui.

/// Mail refers to the idea of a mailing service.
///
/// In the scope of snui, a Mail is an interface widgets can use exchange messages with the application.
/// This allows widgets to operate idependently from the application and keep most of the complexity inside the trait implementation.
///
/// M: The subject of your message.
/// Your post will use this type to identify what it should do with your message.
///
/// D: The data or the content of your message.
/// This is additional data you can attach to your message.
///
/// U: The type you want the Mail to return.
///
/// # Usage
///
/// ```
///	/// This example is inspired by the slider
/// ```
pub trait Mail<M, D, U> {
    fn get(&self, message: M) -> Option<U>;
    fn send(&mut self, message: M, data: D) -> Option<U>;
}

pub trait SimpleMail<M, D, U>: Mail<M, D, U> {
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

/// The heart of snui's application model.
/// When your application needs to be updated, your widget's sync method will be invoked.
///
/// If sync returns true, your widgets will receive a Sync event along with your Data.
pub trait Data {
    fn sync(&mut self) -> bool;
}
