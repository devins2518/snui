#[derive(Clone, Debug)]
pub enum Data<'d, R> {
    Null,
    Int(i32),
    Byte(u8),
    Uint(u32),
    Float(f32),
    Double(f64),
    Boolean(bool),
    Str(&'d str),
    String(String),
    Request(R),
    Any(&'d (dyn std::any::Any + Sync + Send)),
}

// Meant for testing purposes and default
#[derive(Clone, Copy, Debug)]
pub struct DummyController {
    serial: Option<u32>,
}

#[derive(Debug)]
pub struct Message<'m, R>(
    // The u32 is a bitmask
    // Users can create an Enum and alias a bitmask to a value or use a constant
    pub R,
    pub Data<'m, R>,
);

impl<'m, R> Message<'m, R> {
    pub fn new<D: Into<Data<'m, R>>>(request: R, data: D) -> Self {
        Message(request, data.into())
    }
    pub fn request(request: R) -> Self {
        Message(request, ().into())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ControllerError {
    Wait,
    Blocking,
    WrongObject,
    NonBlocking,
    WrongSerial,
    PendingSerial,
}

pub trait Controller<R> {
    // Tells the model all incomming messages are linked
    // The Controller returns a token that can be used to deserialize
    fn serialize(&mut self, msg: Message<R>) -> Result<u32, ControllerError>;
    // Ends the serialization
    fn deserialize(&mut self, token: u32) -> Result<(), ControllerError>;
    // These interface are from the pov of the widgets
    fn get<'c>(&'c self, msg: Message<R>) -> Result<Data<'c, R>, ControllerError>;
    // The Message must be a u32 serial.
    fn send<'c>(&'c mut self, msg: Message<R>) -> Result<Data<'c, R>, ControllerError>;
    fn request<'c>(&'c mut self, request: R) -> Result<Data<'c, R>, ControllerError> {
        self.send(Message::new(request, ()))
    }
    // Returns an Ok(Message) if the application needs to be synced
    fn sync(&mut self) -> Result<Message<'static, R>, ControllerError>;
}

impl<'d, R> From<u8> for Data<'d, R> {
    fn from(byte: u8) -> Self {
        Data::Byte(byte)
    }
}

impl<'d, R> From<u32> for Data<'d, R> {
    fn from(uint: u32) -> Self {
        Data::Uint(uint)
    }
}

impl<'d, R> From<usize> for Data<'d, R> {
    fn from(usize: usize) -> Self {
        Data::Uint(usize as u32)
    }
}

impl<'d, R> From<String> for Data<'d, R> {
    fn from(string: String) -> Self {
        Data::String(string)
    }
}

impl<'d, R> From<i32> for Data<'d, R> {
    fn from(int: i32) -> Self {
        Data::Int(int)
    }
}

impl<'d, R> From<&'d str> for Data<'d, R> {
    fn from(s: &'d str) -> Self {
        Data::Str(s)
    }
}

impl<'d, R> From<&'d String> for Data<'d, R> {
    fn from(s: &'d String) -> Self {
        Data::Str(s)
    }
}

impl<'d, R> From<bool> for Data<'d, R> {
    fn from(b: bool) -> Self {
        Data::Boolean(b)
    }
}

impl<'d, R> From<f32> for Data<'d, R> {
    fn from(f: f32) -> Self {
        Data::Float(f)
    }
}

impl<'d, R> From<f64> for Data<'d, R> {
    fn from(f: f64) -> Self {
        Data::Double(f)
    }
}

impl<'d, R> From<()> for Data<'d, R> {
    fn from(_: ()) -> Self {
        Data::Null
    }
}

impl<'d, R> From<&'d (dyn std::any::Any + Sync + Send)> for Data<'d, R> {
    fn from(any: &'d (dyn std::any::Any + Sync + Send)) -> Self {
        Data::Any(any)
    }
}

// Always returns false for Any
impl<'d, R: PartialEq> PartialEq for Data<'d, R> {
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
            _ => {}
        }
        false
    }
}

impl<'d, R> ToString for Data<'d, R> {
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
            Data::Request(_) => panic!(
                "{} cannot be formatted into a string.",
                std::any::type_name::<R>()
            ),
            Data::Any(_) => panic!("Any cannot be formatted into a string."),
        }
    }
}

impl<'d, R: ToString> Data<'d, R> {
    pub fn to_string(&self) -> String {
        match self {
            Data::Request(request) => request.to_string(),
            Data::Any(_) => panic!("Any cannot be formatted into a string."),
            _ => self.to_string(),
        }
    }
}

impl<'d, R: PartialEq> Eq for Data<'d, R> {}

/*
 * Barebone implementation of Controller.
 * Can be used for debugging your application.
 */
impl DummyController {
    pub fn new() -> Self {
        DummyController { serial: None }
    }
}

impl Controller<()> for DummyController {
    fn serialize(&mut self, _msg: Message<()>) -> Result<u32, ControllerError> {
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
    fn get<'c>(&'c self, msg: Message<()>) -> Result<Data<'c, ()>, ControllerError> {
        println!("<- {:?}", msg.1);
        println!("-> Null");
        Err(ControllerError::WrongObject)
    }
    fn send<'c>(&'c mut self, msg: Message<()>) -> Result<Data<'c, ()>, ControllerError> {
        if let Some(serial) = &self.serial {
            println!("<- {} : {:?}", serial, msg.1);
        } else {
            println!("<- {:?}", msg.1);
        }
        Err(ControllerError::WrongObject)
    }
    fn sync(&mut self) -> Result<Message<'static, ()>, ControllerError> {
        Err(ControllerError::NonBlocking)
    }
}
