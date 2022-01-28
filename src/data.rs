/// Create a new message from a template (&self) and a given parameter (T).
pub trait TryFromArg<T>
where
    Self: Sized,
{
    type Error;
    fn try_from_arg(&self, _: T) -> Result<Self, Self::Error>;
}

/// Create a new message from a template (&self) and a given parameter (T).
pub trait FromArg<T>
where
    Self: Sized,
{
    fn from_arg(&self, _: T) -> Self;
}

pub trait Message<M, D, U> {
    fn get(&self, message: M) -> Option<U>;
    fn send(&mut self, message: M, data: D) -> Option<U>;
}

pub trait SimpleMessage<M, D, U>: Message<M, D, U> {
    fn send(&mut self, message: M) -> Option<U>;
}

impl<M, U, T> SimpleMessage<M, (), U> for T
where
    T: Message<M, (), U>
{
    fn send(&mut self, message: M) -> Option<U> {
        Message::send(self, message, ())
    }
}

impl<I, T> TryFromArg<T> for I
where
    I: FromArg<T>,
{
    type Error = ();
    fn try_from_arg(&self, t: T) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(FromArg::from_arg(self, t))
    }
}

/// The heart of snui's application model.
/// When your application needs to be updated, your widget's sync method will be invoked
/// and your Data will be propagates down the widget tree.
pub trait Data {
    fn sync(&mut self) -> bool;
}
