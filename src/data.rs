#[derive(Clone, Copy, Debug)]
pub enum Data<'d> {
    Null,
    Int(i32),
    Uint(u32),
    Float(f32),
    Boolean(bool),
    String(&'d str),
    Any(&'d dyn std::any::Any),
}

// Meant for testing purposes and default
#[derive(Clone, Copy, Debug)]
pub struct DummyModel {
    serial: Option<u32>,
}

#[derive(Clone, Copy, Debug)]
pub struct Message<'m>(
    // The u32 is a bitmask
    // Users can create an Enum and alias a bitmask to a value
    u32,
    Data<'m>,
);

impl<'m> Message<'m> {
    pub fn new<D: Into<Data<'m>>>(obj: u32, data: D) -> Self {
        Message ( obj, data.into() )
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ModelError {
    Block,
    WrongSerial,
    PendingSerial,
}

pub trait Model {
    // Tells the model all incomming messages are linked
    // The Model returns a token that can be used to deserialize
    fn serialize(&mut self, msg: Message) -> Result<u32, ModelError>;
    // Ends the serialization
    fn deserialize(&mut self, token: u32) -> Result<(), ModelError>;
    // These interface are from the pov
    // of the widgets
    fn get<'m>(&'m self, msg: Message) -> Result<Data<'m>, ModelError>;
    // The Message must be a u32 serial.
    fn send<'m>(&'m mut self, msg: Message) -> Result<Data<'m>, ModelError>;
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

impl DummyModel {
    pub fn new() -> Self {
        DummyModel { serial: None }
    }
}

impl Model for DummyModel {
    fn serialize(&mut self, msg: Message) -> Result<u32, ModelError> {
        if self.serial.is_some() {
            return Err(ModelError::PendingSerial);
        } else {
            self.serial = Some(1)
        }
        println!("Serialize Token: {}", 1);
        Ok(1)
    }
    fn deserialize(&mut self, token: u32) -> Result<(), ModelError> {
        if let Some(serial) = self.serial {
            if serial != token {
                return Err(ModelError::WrongSerial);
            } else {
                println!("Deserialize: {}", 1);
                self.serial = None;
            }
        }
        Ok(())
    }
    fn get<'m>(&'m self, msg: Message) -> Result<Data<'m>, ModelError> {
        if self.serial.is_some() {
            return Err(ModelError::PendingSerial)
        }
        println!("{:?}", msg);
        Ok(Data::Null)
    }
    fn send<'m>(&'m mut self, msg: Message) -> Result<Data<'m>, ModelError> {
        println!("{:?}", msg);
        Ok(Data::Null)
    }
}
