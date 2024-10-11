// 引入必要的外部crate
use serde::Deserialize; // 用于从TOML格式反序列化结构体
use std::fs; // 用于文件系统操作，如读取文件

// 定义主要的Config结构体
// #[derive(Deserialize)]属性允许这个结构体从TOML格式自动反序列化
#[derive(Deserialize)]
pub struct Config {
    pub server: ServerConfig, // 包含服务器相关配置的嵌套结构
    pub client: ClientConfig, // 包含客户端相关配置的嵌套结构
}

// 定义ServerConfig结构体，包含服务器特定的配置项
#[derive(Deserialize)]
pub struct ServerConfig {
    pub address: String, // 服务器地址，如 "127.0.0.1:8080"
    pub loss_rate: f32,  // 模拟网络丢包率，范围 0.0 到 1.0
}

// 定义ClientConfig结构体，包含客户端特定的配置项
#[derive(Deserialize)]
pub struct ClientConfig {
    pub timeout: u32,    // 客户端请求超时时间，单位可能是秒或毫秒
    pub retry: u32,      // 客户端重试次数
    pub invocation_semantic: String, // 调用语义，可能的值如 "at-least-once", "at-most-once" 等
}

// 为Config结构体实现方法
impl Config {
    // 静态方法：加载配置
    // 返回Result类型，成功时包含Config实例，失败时包含错误
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // 从"config.toml"文件读取内容
        // ?运算符在遇到错误时会立即返回错误，简化了错误处理
        let config_text = fs::read_to_string("config.toml")?;
        
        // 使用toml crate将文本解析为Config结构体
        // 如果解析失败，?运算符会返回错误
        let config: Config = toml::from_str(&config_text)?;
        
        // 如果一切正常，返回解析后的Config实例
        Ok(config)
    }
}

// main函数，目前未使用
// #[allow(dead_code)]属性防止编译器对未使用的函数发出警告
#[allow(dead_code)]
fn main() {
    // 主函数目前为空
    // 这里可以添加代码来测试Config的加载和使用，例如：
    // match Config::load() {
    //     Ok(config) => println!("配置加载成功: {:?}", config),
    //     Err(e) => eprintln!("配置加载失败: {}", e),
    // }
}

