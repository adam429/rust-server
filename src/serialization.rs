use std::collections::HashMap;
use std::fmt;
use std::io::{Cursor, Read, Write};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

/// Represents the byte order for serialization and deserialization.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum ByteOrder {
    Big,
    Little,
}

/// Represents the supported data types for serialization and deserialization.
#[derive(Debug, Clone, Copy)]
pub enum DataType {
    Int32,
    Bool,
    String,
    Float,
    Array,
    Map,
}

impl DataType {
    /// Converts the DataType to its corresponding u8 representation.
    fn to_u8(self) -> u8 {
        match self {
            DataType::Int32 => 1,
            DataType::Bool => 2,
            DataType::String => 3,
            DataType::Float => 4,
            DataType::Array => 5,
            DataType::Map => 6,
        }
    }

    /// Converts a u8 value to its corresponding DataType, if valid.
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(DataType::Int32),
            2 => Some(DataType::Bool),
            3 => Some(DataType::String),
            4 => Some(DataType::Float),
            5 => Some(DataType::Array),
            6 => Some(DataType::Map),
            _ => None,
        }
    }
}

/// Handles the serialization of data into a byte buffer.
pub struct Serializer {
    buffer: Vec<u8>,
    byte_order: ByteOrder,
}

impl Serializer {
    /// Creates a new Serializer with the specified byte order.
    pub fn new(byte_order: ByteOrder) -> Self {
        Serializer {
            buffer: Vec::new(),
            byte_order,
        }
    }

    /// Writes the data type to the buffer.
    fn write_type(&mut self, data_type: DataType) -> std::io::Result<()> {
        self.buffer.write_u8(data_type.to_u8())
    }

    /// Serializes an i32 value.
    pub fn serialize_int32(&mut self, value: i32) -> std::io::Result<()> {
        self.write_type(DataType::Int32)?;
        match self.byte_order {
            ByteOrder::Big => self.buffer.write_i32::<BigEndian>(value),
            ByteOrder::Little => self.buffer.write_i32::<LittleEndian>(value),
        }
    }

    /// Serializes a boolean value.
    pub fn serialize_bool(&mut self, value: bool) -> std::io::Result<()> {
        self.write_type(DataType::Bool)?;
        self.buffer.write_u8(if value { 1 } else { 0 })
    }

    /// Serializes a string value.
    pub fn serialize_string(&mut self, value: &str) -> std::io::Result<()> {
        self.write_type(DataType::String)?;
        self.serialize_int32(value.len() as i32)?;
        self.buffer.write_all(value.as_bytes())
    }

    /// Serializes a f32 value.
    pub fn serialize_float(&mut self, value: f32) -> std::io::Result<()> {
        self.write_type(DataType::Float)?;
        match self.byte_order {
            ByteOrder::Big => self.buffer.write_f32::<BigEndian>(value),
            ByteOrder::Little => self.buffer.write_f32::<LittleEndian>(value),
        }
    }

    /// Serializes an array of serializable items.
    pub fn serialize_array<T: Serialize>(&mut self, array: &[T]) -> std::io::Result<()> {
        self.write_type(DataType::Array)?;
        self.serialize_int32(array.len() as i32)?;
        for item in array {
            item.serialize(self)?;
        }
        Ok(())
    }    

    /// Serializes a map of serializable keys and values.
    pub fn serialize_map<K: Serialize, V: Serialize>(&mut self, map: &HashMap<K, V>) -> std::io::Result<()> {
        map.serialize(self)
    }

    /// Returns the serialized buffer.
    pub fn get_buffer(self) -> Vec<u8> {
        self.buffer
    }
}

/// Trait for types that can be serialized.
pub trait Serialize {
    fn serialize(&self, serializer: &mut Serializer) -> std::io::Result<()>;
}

// Implement Serialize for various primitive types
impl Serialize for i32 {
    fn serialize(&self, serializer: &mut Serializer) -> std::io::Result<()> {
        serializer.serialize_int32(*self)
    }
}

impl Serialize for f32 {
    fn serialize(&self, serializer: &mut Serializer) -> std::io::Result<()> {
        serializer.serialize_float(*self)
    }
}

impl Serialize for String {
    fn serialize(&self, serializer: &mut Serializer) -> std::io::Result<()> {
        serializer.serialize_string(self)
    }
}

impl Serialize for &str {
    fn serialize(&self, serializer: &mut Serializer) -> std::io::Result<()> {
        serializer.serialize_string(self)
    }
}

impl Serialize for bool {
    fn serialize(&self, serializer: &mut Serializer) -> std::io::Result<()> {
        serializer.serialize_bool(*self)
    }
}

impl<K, V> Serialize for HashMap<K, V>
where
    K: Serialize,
    V: Serialize,
{
    fn serialize(&self, serializer: &mut Serializer) -> std::io::Result<()> {
        serializer.write_type(DataType::Map)?;
        serializer.serialize_int32(self.len() as i32)?;
        for (key, value) in self {
            key.serialize(serializer)?;
            value.serialize(serializer)?;
        }
        Ok(())
    }
}

/// Handles the deserialization of data from a byte buffer.
pub struct Deserializer<'a> {
    cursor: Cursor<&'a [u8]>,
    byte_order: ByteOrder,
}

impl<'a> Deserializer<'a> {
    /// Creates a new Deserializer with the given buffer and byte order.
    pub fn new(buffer: &'a [u8], byte_order: ByteOrder) -> Self {
        Deserializer {
            cursor: Cursor::new(buffer),
            byte_order,
        }
    }

    /// Reads the data type from the buffer.
    fn read_type(&mut self) -> std::io::Result<DataType> {
        let type_byte = self.cursor.read_u8()?;
        DataType::from_u8(type_byte)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data type"))
    }

    /// Deserializes the next value from the buffer.
    pub fn deserialize_next(&mut self) -> std::io::Result<Value> {
        let data_type = self.read_type()?;
        match data_type {
            DataType::Int32 => Ok(Value::Int32(self.deserialize_int32()?)),
            DataType::Bool => Ok(Value::Bool(self.deserialize_bool()?)),
            DataType::String => Ok(Value::String(self.deserialize_string()?)),
            DataType::Float => Ok(Value::Float(self.deserialize_float()?)),
            DataType::Array => {
                let array = self.deserialize_array(|d| d.deserialize_next())?;
                Ok(Value::Array(array))
            }
            DataType::Map => {
                let map = self.deserialize_map(
                    |d| d.deserialize_next().and_then(|v| v.into_string()),
                    |d| d.deserialize_next(),
                )?;
                Ok(Value::Map(map))
            }
        }
    }

    /// Deserializes an i32 value.
    pub fn deserialize_int32(&mut self) -> std::io::Result<i32> {
        match self.byte_order {
            ByteOrder::Big => self.cursor.read_i32::<BigEndian>(),
            ByteOrder::Little => self.cursor.read_i32::<LittleEndian>(),
        }
    }

    /// Deserializes a boolean value.
    pub fn deserialize_bool(&mut self) -> std::io::Result<bool> {
        Ok(self.cursor.read_u8()? != 0)
    }

    /// Deserializes a string value.
    pub fn deserialize_string(&mut self) -> std::io::Result<String> {
        self.cursor.set_position(self.cursor.position() + 1);
        let len = self.deserialize_int32()? as usize;
        let mut buffer = vec![0u8; len];
        self.cursor.read_exact(&mut buffer)?;
        String::from_utf8(buffer).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Deserializes a f32 value.
    pub fn deserialize_float(&mut self) -> std::io::Result<f32> {
        match self.byte_order {
            ByteOrder::Big => self.cursor.read_f32::<BigEndian>(),
            ByteOrder::Little => self.cursor.read_f32::<LittleEndian>(),
        }
    }

    /// Deserializes an array of items.
    pub fn deserialize_array<T, F>(&mut self, deserialize_item: F) -> std::io::Result<Vec<T>>
    where
        F: Fn(&mut Self) -> std::io::Result<T>,
    {   
        self.cursor.set_position(self.cursor.position() + 1);
        let len = self.deserialize_int32()? as usize;
        let mut array = Vec::with_capacity(len);
        for _ in 0..len {
            array.push(deserialize_item(self)?);
        }
        Ok(array)
    }

    /// Deserializes a map of key-value pairs.
    pub fn deserialize_map<K, V, FK, FV>(
        &mut self,
        deserialize_key: FK,
        deserialize_value: FV,
    ) -> std::io::Result<HashMap<K, V>>
    where
        K: std::hash::Hash + Eq,
        FK: Fn(&mut Self) -> std::io::Result<K>,
        FV: Fn(&mut Self) -> std::io::Result<V>,
    {
        self.cursor.set_position(self.cursor.position() + 1);
        let len = self.deserialize_int32()? as usize;
        let mut map = HashMap::with_capacity(len);
        for _ in 0..len {
            let key = deserialize_key(self)?;
            let value = deserialize_value(self)?;
            map.insert(key, value);
        }
        Ok(map)
    }
}

/// Represents a deserialized value.
#[derive(Debug)]
pub enum Value {
    Int32(i32),
    Bool(bool),
    String(String),
    Float(f32),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int32(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::Array(v) => {
                write!(f, "[")?;
                for (i, item) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            },
            Value::Map(m) => {
                write!(f, "{{")?;
                for (i, (key, value)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, "}}")
            },
        }
    }
}

#[allow(dead_code)]
impl Value {
    /// Returns the value as an i32 if it is an Int32, otherwise None.
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Value::Int32(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the value as a bool if it is a Bool, otherwise None.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns a reference to the String if it is a String, otherwise None.
    pub fn as_string(&self) -> Option<&String> {
        match self {
            Value::String(v) => Some(v),
            _ => None,
        }
    }

    /// Returns the value as an f32 if it is a Float, otherwise None.
    pub fn as_float(&self) -> Option<f32> {
        match self {
            Value::Float(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns a reference to the Vec<Value> if it is an Array, otherwise None.
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(v) => Some(v),
            _ => None,
        }
    }

    /// Returns a reference to the HashMap<String, Value> if it is a Map, otherwise None.
    pub fn as_map(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Map(v) => Some(v),
            _ => None,
        }
    }

    /// Converts the Value into a String if it is a String, otherwise returns an error.
    fn into_string(self) -> std::io::Result<String> {
        if let Value::String(s) = self {
            Ok(s)
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Value is not a String"))
        }
    }
}