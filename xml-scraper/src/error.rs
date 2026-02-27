//! 错误类型定义

use std::fmt;

/// xml-scraper 库的错误类型
#[derive(Debug, Clone)]
pub enum Error {
    /// 输入包含空字节
    NullByte,
    /// HTML/XML 解析失败
    ParseFailed,
    /// XPath 表达式无效
    InvalidXPath,
    /// 创建 XPath 上下文失败
    XPathContextFailed,
    /// 自定义错误消息
    Custom(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NullByte => write!(f, "Input contains null byte"),
            Error::ParseFailed => write!(f, "Failed to parse HTML/XML"),
            Error::InvalidXPath => write!(f, "Invalid XPath expression"),
            Error::XPathContextFailed => write!(f, "Failed to create XPath context"),
            Error::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error::Custom(s.to_string())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Custom(s)
    }
}
