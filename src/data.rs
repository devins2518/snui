#[derive(Debug, Clone, Copy)]
pub enum Data<'d> {
    Null,
    Int(i32),
    Byte(u8),
    Uint(u32),
    Float(f32),
    Double(f64),
    Boolean(bool),
    String(&'d str),
    Any(&'d dyn std::any::Any),
}

// Meant for testing purposes and default
#[derive(Clone, Copy, Debug)]
pub struct DummyController {
    serial: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Message<'m>(
    // The u32 is a bitmask
    // Users can create an Enum and alias a bitmask to a value or use a constant
    pub u32,
    pub Data<'m>,
);

impl<'m> Message<'m> {
    pub fn new<D: Into<Data<'m>>>(obj: u32, data: D) -> Self {
        Message(obj, data.into())
    }
}

// Returns a message with object 0 and Data::Null
impl<'m> Default for Message<'m> {
    fn default() -> Self {
        Message(0, Data::Null)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ControllerError {
    Block,
    WrongObject,
    NonBlocking,
    WrongSerial,
    PendingSerial,
}

pub trait Controller {
    // Tells the model all incomming messages are linked
    // The Controller returns a token that can be used to deserialize
    fn serialize(&mut self, msg: Message) -> Result<u32, ControllerError>;
    // Ends the serialization
    fn deserialize(&mut self, token: u32) -> Result<(), ControllerError>;
    // These interface are from the pov of the widgets
    fn get<'m>(&'m self, msg: Message) -> Result<Data<'m>, ControllerError>;
    fn request<'m>(&'m self, obj: u32) -> Result<Data<'m>, ControllerError> {
        self.get(Message::new(obj, Data::Null))
    }
    // The Message must be a u32 serial.
    fn send<'m>(&'m mut self, msg: Message) -> Result<Data<'m>, ControllerError>;
    // Returns an Ok(u32) if the application needs to be synced
    fn sync(&mut self) -> Result<Message<'static>, ControllerError>;
}

impl<'d> From<u8> for Data<'d> {
    fn from(byte: u8) -> Self {
        Data::Byte(byte)
    }
}

impl<'d> From<u32> for Data<'d> {
    fn from(uint: u32) -> Self {
        Data::Uint(uint)
    }
}

impl<'d> From<i32> for Data<'d> {
    fn from(int: i32) -> Self {
        Data::Int(int)
    }
}

impl<'d> From<&'d str> for Data<'d> {
    fn from(s: &'d str) -> Self {
        Data::String(s)
    }
}

impl<'d> From<bool> for Data<'d> {
    fn from(b: bool) -> Self {
        Data::Boolean(b)
    }
}

impl<'d> From<f32> for Data<'d> {
    fn from(f: f32) -> Self {
        Data::Float(f)
    }
}

impl<'d> From<f64> for Data<'d> {
    fn from(f: f64) -> Self {
        Data::Double(f)
    }
}

impl<'d> From<&'d dyn std::any::Any> for Data<'d> {
    fn from(any: &'d dyn std::any::Any) -> Self {
        Data::Any(any)
    }
}

// Always returns false for Any
impl<'d> PartialEq for Data<'d> {
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
            _ => {}
        }
        false
    }
}

impl<'d> Eq for Data<'d> {}

/*
 * Barebone implementation of Controller.
 * Can be used for debugging your application.
 */
impl DummyController {
    pub fn new() -> Self {
        DummyController { serial: None }
    }
}

impl Controller for DummyController {
    fn serialize(&mut self, _msg: Message) -> Result<u32, ControllerError> {
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
    fn get<'m>(&'m self, msg: Message) -> Result<Data<'m>, ControllerError> {
        println!("<- {:?}", msg);
        println!("-> Null");
        Err(ControllerError::WrongObject)
    }
    fn send<'m>(&'m mut self, msg: Message) -> Result<Data<'m>, ControllerError> {
        if let Some(serial) = &self.serial {
            println!("<- {} : {:?}", serial, msg);
        } else {
            println!("<- {:?}", msg);
        }
        Err(ControllerError::WrongObject)
    }
    fn sync(&mut self) -> Result<Message<'static>, ControllerError> {
        Err(ControllerError::NonBlocking)
    }
}
