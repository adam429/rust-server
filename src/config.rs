// 引入必要的外部crate
use serde::Deserialize; // 用于反序列化
use std::fs; // 用于文件系统操作

// 定义主要的Config结构体
// #[derive(Deserialize)]属性允许这个结构体从TOML格式反序列化
#[derive(Deserialize)]
pub struct Config {
    pub server: ServerConfig, // 包含服务器配置的嵌套结构
}

// 定义ServerConfig结构体
// 同样使用#[derive(Deserialize)]以允许从TOML反序列化
#[derive(Deserialize)]
pub struct ServerConfig {
    pub address: String, // 服务器地址,作为字符串存储
}

// 为Config结构体实现方法
impl Config {
    // 加载配置的静态方法
    // 返回Result类型,成功时包含Config实例,失败时包含错误
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // 从"config.toml"文件读取内容
        // ?运算符在遇到错误时会提前返回错误
        let config_text = fs::read_to_string("config.toml")?;
        
        // 使用toml crate将文本解析为Config结构体
        // 同样使用?运算符处理可能的错误
        let config: Config = toml::from_str(&config_text)?;
        
        // 如果一切正常,返回解析后的Config实例
        Ok(config)
    }
}

// main函数,目前未使用
// #[allow(dead_code)]属性防止编译器对未使用的函数发出警告
#[allow(dead_code)]
fn main() {
    // 主函数目前为空
    // 可以在这里添加代码来测试Config的加载和使用
}

// 注意: 这个文件假定在同一目录下存在一个名为"config.toml"的配置文件
// 该文件应包含与Config和ServerConfig结构体匹配的TOML格式数据
