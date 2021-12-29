#[derive(Clone, Debug)]
pub enum Data<'d, M> {
    Null,
    Int(i32),
    Byte(u8),
    Uint(u32),
    Float(f32),
    Double(f64),
    Boolean(bool),
    Str(&'d str),
    String(String),
    Message(M),
    MsgRef(&'d M),
    Any(&'d (dyn std::any::Any + Sync + Send)),
}

pub trait TryIntoMessage<T> {
    type Error;
    fn into(&self, _: T) -> Result<Self, Self::Error> where Self : Sized;
}

pub trait IntoMessage<T>
where
    Self: Sized
{
    fn into(&self, _: T) -> Self;
}

impl<I, T> TryIntoMessage<T> for I
where
    Self: IntoMessage<T>
{
    type Error = ();
    fn into(&self, t: T) -> Result<Self, Self::Error>
    where Self : Sized {
        Ok(IntoMessage::into(self, t))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControllerError {
    Waiting,
    Blocking,
    Message,
    WrongSerial,
    PendingSerial,
}

pub trait Controller<M> {
    // Tells the model all incomming messages are linked
    // The Controller returns a token that can be used to deserialize
    fn serialize(&mut self) -> Result<u32, ControllerError>;
    // Ends the serialization
    fn deserialize(&mut self, token: u32) -> Result<(), ControllerError>;
    // These interface are from the pov of the widgets
    fn get<'m>(&'m self, msg: &'m M) -> Result<Data<'m, M>, ControllerError>;
    // The Message must be a u32 serial.
    fn send<'m>(&'m mut self, msg: M) -> Result<Data<'m, M>, ControllerError>;
    // Returns an Ok(Message) if the application needs to be synced
    fn sync(&mut self) -> Result<M, ControllerError>;
}

impl<'d, M> From<u8> for Data<'d, M> {
    fn from(byte: u8) -> Self {
        Data::Byte(byte)
    }
}

impl<'d, M> From<u32> for Data<'d, M> {
    fn from(uint: u32) -> Self {
        Data::Uint(uint)
    }
}

impl<'d, M> From<usize> for Data<'d, M> {
    fn from(usize: usize) -> Self {
        Data::Uint(usize as u32)
    }
}

impl<'d, M> From<String> for Data<'d, M> {
    fn from(string: String) -> Self {
        Data::String(string)
    }
}

impl<'d, M> From<i32> for Data<'d, M> {
    fn from(int: i32) -> Self {
        Data::Int(int)
    }
}

impl<'d, M> From<&'d str> for Data<'d, M> {
    fn from(s: &'d str) -> Self {
        Data::Str(s)
    }
}

impl<'d, M> From<&'d String> for Data<'d, M> {
    fn from(s: &'d String) -> Self {
        Data::Str(s)
    }
}

impl<'d, M> From<bool> for Data<'d, M> {
    fn from(b: bool) -> Self {
        Data::Boolean(b)
    }
}

impl<'d, M> From<f32> for Data<'d, M> {
    fn from(f: f32) -> Self {
        Data::Float(f)
    }
}

impl<'d, M> From<f64> for Data<'d, M> {
    fn from(f: f64) -> Self {
        Data::Double(f)
    }
}

impl<'d, M> From<()> for Data<'d, M> {
    fn from(_: ()) -> Self {
        Data::Null
    }
}

// Always returns false for Any
impl<'d, M: PartialEq> PartialEq for Data<'d, M> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Data::Boolean(s) => {
                if let Data::Boolean(o) = other {
                    return s == o;
                }
            }
            Data::Uint(s) => {
                if let Data::Uint(o) = other {
                    return s == o;
                }
            }
            Data::Int(s) => {
                if let Data::Int(o) = other {
                    return s == o;
                }
            }
            Data::String(s) => {
                if let Data::String(o) = other {
                    return s == o;
                }
            }
            Data::Str(s) => {
                if let Data::Str(o) = other {
                    return s == o;
                }
            }
            Data::Byte(s) => {
                if let Data::Byte(o) = other {
                    return s == o;
                }
            }
            Data::Double(s) => {
                if let Data::Double(o) = other {
                    return s == o;
                }
            }
            Data::Float(s) => {
                if let Data::Float(o) = other {
                    return s == o;
                }
            }
            Data::Message(s) => {
                if let Data::Message(o) = other {
                    return s == o;
                }
            }
            Data::MsgRef(s) => {
                if let Data::MsgRef(o) = other {
                    return s == o;
                }
            }
            _ => {}
        }
        false
    }
}

impl<'d, M> ToString for Data<'d, M> {
    fn to_string(&self) -> String {
        match self {
            Data::Boolean(b) => b.to_string(),
            Data::Uint(u) => u.to_string(),
            Data::Int(i) => i.to_string(),
            Data::String(s) => s.to_string(),
            Data::Str(s) => s.to_string(),
            Data::Byte(b) => b.to_string(),
            Data::Double(d) => d.to_string(),
            Data::Float(f) => f.to_string(),
            Data::Null => String::new(),
            Data::Message(_) => panic!(
                "{} cannot be formatted into a string.",
                std::any::type_name::<M>()
            ),
            Data::MsgRef(_) => panic!(
                "{} cannot be formatted into a string.",
                std::any::type_name::<M>()
            ),
            Data::Any(_) => panic!("Any cannot be formatted into a string."),
        }
    }
}

impl<'d, M: ToString> Data<'d, M> {
    pub fn to_string(&self) -> String {
        match self {
            Data::Message(request) => request.to_string(),
            Data::MsgRef(request) => request.to_string(),
            Data::Any(_) => panic!("Any cannot be formatted into a string."),
            _ => self.to_string(),
        }
    }
}

impl<'d, M: PartialEq> Eq for Data<'d, M> {}

// Meant for testing purposes and default
#[derive(Clone, Copy, Debug)]
pub struct DummyController<M>
where
    M: std::fmt::Debug
{
    serial: Option<u32>,
    data: M
}

/*
 * Barebone implementation of Controller.
 * Can be used for debugging your application.
 */
impl<M> DummyController<M>
where
    M: std::fmt::Debug
{
    pub fn new(data: M) -> Self {
        DummyController { serial: None, data }
    }
}

impl<M> Controller<M> for DummyController<M>
where
    M: std::fmt::Debug
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
    fn deserialize(&mut self, token: u32) -> Result<(), ControllerError> {
        if let Some(serial) = self.serial {
            if serial != token {
                return Err(ControllerError::WrongSerial);
            } else {
                println!("Deserialize: {}", 1);
                self.serial = None;
            }
        }
        Ok(())
    }
    fn get<'m>(&'m self, msg: &'m M) -> Result<Data<'m, M>, ControllerError> {
        println!("<- {:?}", msg);
        Ok(Data::Null)
    }
    fn send<'m>(&'m mut self, msg: M) -> Result<Data<'m, M>, ControllerError> {
        if let Some(serial) = &self.serial {
            println!("<- {} : {:?}", serial, msg);
        } else {
            println!("<- {:?}", msg);
        }
        self.data = msg;
        Err(ControllerError::Message)
    }
    fn sync(&mut self) -> Result<M, ControllerError> {
        Err(ControllerError::Waiting)
    }
}
