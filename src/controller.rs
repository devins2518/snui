pub trait TryIntoMessage<T>
where
    Self: Sized,
{
    type Error;
    fn try_into(&self, _: T) -> Result<Self, Self::Error>;
}

pub trait IntoMessage<T>
where
    Self: Sized,
{
    fn into(&self, _: T) -> Self;
}

impl<I, T> TryIntoMessage<T> for I
where
    Self: IntoMessage<T>,
{
    type Error = ();
    fn try_into(&self, t: T) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(IntoMessage::into(self, t))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControllerError {
    Waiting,
    Blocking,
    Message,
    Data,
    WrongSerial,
    PendingSerial,
    NonSerialized,
}

pub trait Controller<M> {
    // Tells the model all incomming messages are linked
    // The Controller returns a token that can be used to deserialize
    fn serialize(&mut self) -> Result<u32, ControllerError> {
        Err(ControllerError::NonSerialized)
    }
    // Ends the serialization
    fn deserialize(&mut self, _serial: u32) -> Result<(), ControllerError> {
        Err(ControllerError::NonSerialized)
    }
    // These interface are from the pov of the widgets
    fn get(&self, msg: &M) -> Result<M, ControllerError>;
    fn send(&mut self, msg: M) -> Result<M, ControllerError>;
    fn send_serialize(&mut self, _serial: u32, _msg: M) -> Result<M, ControllerError> {
        Err(ControllerError::NonSerialized)
    }
    // Returns an Ok(Message) if the application needs to be synced
    fn sync<'s>(&mut self) -> Result<M, ControllerError>;
}

// Meant for testing purposes and default
#[derive(Clone, Copy, Debug)]
pub struct DummyController<M>
where
    M: std::fmt::Debug,
{
    serial: Option<u32>,
    data: M,
}

/*
 * Barebone implementation of Controller.
 * Can be used for debugging your application.
 */
impl<M> DummyController<M>
where
    M: std::fmt::Debug,
{
    pub fn new(data: M) -> Self {
        DummyController { serial: None, data }
    }
}

impl<M> Controller<M> for DummyController<M>
where
    M: std::fmt::Debug,
{
    fn serialize(&mut self) -> Result<u32, ControllerError> {
        if self.serial.is_some() {
            return Err(ControllerError::PendingSerial);
        } else {
            self.serial = Some(1)
        }
        println!("Serialize Token: {}", 1);
        Ok(1)
    }
    fn deserialize(&mut self, serial: u32) -> Result<(), ControllerError> {
        if let Some(this) = self.serial {
            if this != serial {
                return Err(ControllerError::WrongSerial);
            } else {
                println!("Deserialize: {}", 1);
                self.serial = None;
            }
        }
        Ok(())
    }
    fn get<'m>(&'m self, msg: &'m M) -> Result<M, ControllerError> {
        println!("<- {:?}", msg);
        Err(ControllerError::Message)
    }
    fn send(&mut self, msg: M) -> Result<M, ControllerError> {
        self.data = msg;
        Err(ControllerError::Message)
    }
    fn send_serialize<'m>(&'m mut self, serial: u32, msg: M) -> Result<M, ControllerError> {
        if Some(serial) == self.serial {
            println!("<- {} : {:?}", serial, msg);
        } else {
            println!("<- {:?}", msg);
        }
        Err(ControllerError::Message)
    }
    fn sync<'s>(&mut self) -> Result<M, ControllerError> {
        Err(ControllerError::Waiting)
    }
}
