//! 错误类型定义

use thiserror::Error;

/// xml-scraper 库的错误类型
#[derive(Debug, Clone, Error)]
pub enum Error {
    /// 输入包含空字节
    ///
    /// C 字符串不能包含空字节，请检查输入数据。
    #[error("Input contains null byte")]
    NullByte,

    /// 输入数据过大
    ///
    /// 输入大小超过了 i32::MAX 字节限制。
    #[error("Input too large: {size} bytes (max: {max})")]
    InputTooLarge {
        /// 实际大小（字节）
        size: usize,
        /// 最大允许大小
        max: usize,
    },

    /// HTML/XML 解析失败
    ///
    /// 输入数据无法被正确解析。使用容错模式可能有所帮助。
    #[error("Failed to parse HTML/XML{}", .detail.as_ref().map(|d| format!(": {}", d)).unwrap_or_default())]
    ParseFailed {
        /// 可选的错误详情
        detail: Option<String>,
    },

    /// XPath 表达式无效
    ///
    /// XPath 语法错误或表达式无法求值。
    #[error("Invalid XPath expression '{}'{}", .xpath, .reason.as_ref().map(|r| format!(": {}", r)).unwrap_or_default())]
    InvalidXPath {
        /// XPath 表达式
        xpath: String,
        /// 错误原因
        reason: Option<String>,
    },

    /// 创建 XPath 上下文失败
    ///
    /// 这是一个内部错误，通常表示内存不足。
    #[error("Failed to create XPath context")]
    XPathContextFailed,

    /// 节点不存在
    ///
    /// 尝试访问不存在的节点或属性。
    #[error("Node not found: {node}")]
    NodeNotFound {
        /// 节点描述
        node: String,
    },

    /// 属性不存在
    #[error("Attribute not found: {name}")]
    AttributeNotFound {
        /// 属性名
        name: String,
    },

    /// 文档已释放
    ///
    /// 尝试在文档被释放后访问其内容。
    #[error("Document has been freed")]
    DocumentFreed,

    /// 自定义错误消息
    #[error("{0}")]
    Custom(String),
}

// 保留向后兼容的 From 实现
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

/// 便捷的 Result 类型别名
pub type Result<T> = std::result::Result<T, Error>;

// 实现一些便捷的构造函数
impl Error {
    /// 创建解析失败错误（带详情）
    pub fn parse_failed(detail: impl Into<String>) -> Self {
        Error::ParseFailed {
            detail: Some(detail.into()),
        }
    }

    /// 创建无效 XPath 错误
    pub fn invalid_xpath(xpath: impl Into<String>) -> Self {
        Error::InvalidXPath {
            xpath: xpath.into(),
            reason: None,
        }
    }

    /// 创建无效 XPath 错误（带原因）
    pub fn invalid_xpath_with_reason(xpath: impl Into<String>, reason: impl Into<String>) -> Self {
        Error::InvalidXPath {
            xpath: xpath.into(),
            reason: Some(reason.into()),
        }
    }
}
