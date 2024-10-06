use std::collections::HashMap;
#[path = "../serialization.rs"]
mod serialization;
use serialization::{Serializer, Deserializer, ByteOrder};


fn main() -> std::io::Result<()> {
    let mut serializer = Serializer::new(ByteOrder::Little);

    // 序列化各种类型
    serializer.serialize_int32(42)?;
    serializer.serialize_bool(true)?;
    serializer.serialize_string("Hello, World!")?;
    serializer.serialize_float(3.14)?;

    // 序列化数组
    let int_array = vec![1, 2, 3];
    serializer.serialize_array(&int_array)?;

    let float_array = vec![1.1, 2.2, 3.3];
    serializer.serialize_array(&float_array)?;

    let float_array = vec!["ABC", "DEF", "XYZ"];
    serializer.serialize_array(&float_array)?;

    // 序列化字符串到字符串的映射
    let mut map = HashMap::new();
    map.insert("key1".to_string(), "value1".to_string());
    map.insert("key2".to_string(), "value2".to_string());
    serializer.serialize_map(&map)?;

    let mut map2: HashMap<String, i32> = HashMap::new();
    map2.insert("key1".to_string(), 123);
    map2.insert("key2".to_string(), 456);
    serializer.serialize_map(&map2)?;


    let buffer = serializer.get_buffer();
    println!("Serialized buffer: {:?}", buffer);

    // 将 Vec<u8> 转换为十六进制字符串
    let hex_string: String = buffer.iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    
    println!("Serialized buffer (hex): {}", hex_string);

    // 反序列化
    let mut deserializer = Deserializer::new(&buffer, ByteOrder::Little);

    while let Ok(value) = deserializer.deserialize_next() {
        println!("Deserialized value: {}", value);
    }

    Ok(())
}