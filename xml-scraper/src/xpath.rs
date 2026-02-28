//! XPath 查询与结果处理

use crate::error::{Error, Result};
use crate::node::SelectedNode;
use libxml2_sys::*;
use std::ffi::CString;

/// XPath 查询结果
///
/// XPath 表达式可以返回四种基本类型之一。
///
/// # Example
///
/// ```
/// use xml_scraper::{Document, XPathResult};
///
/// let doc = Document::parse("<div><p>A</p><p>B</p></div>").unwrap();
///
/// // 节点集合
/// match doc.evaluate("//p").unwrap() {
///     XPathResult::NodeSet(nodes) => println!("Found {} nodes", nodes.len()),
///     _ => {}
/// }
///
/// // 数字
/// match doc.evaluate("count(//p)").unwrap() {
///     XPathResult::Number(n) => println!("Count: {}", n),
///     _ => {}
/// }
///
/// // 布尔值
/// match doc.evaluate("count(//p) > 1").unwrap() {
///     XPathResult::Boolean(b) => println!("Has multiple: {}", b),
///     _ => {}
/// }
///
/// // 字符串
/// match doc.evaluate("string(//p)").unwrap() {
///     XPathResult::String(s) => println!("Text: {}", s),
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone)]
pub enum XPathResult<'a> {
    /// 节点集合
    NodeSet(Vec<SelectedNode<'a>>),
    /// 布尔值
    Boolean(bool),
    /// 数字（浮点数）
    Number(f64),
    /// 字符串
    String(std::string::String),
    /// 空结果或未知类型
    Empty,
}

impl<'a> XPathResult<'a> {
    /// 检查是否为节点集合
    #[inline]
    pub fn is_nodeset(&self) -> bool {
        matches!(self, XPathResult::NodeSet(_))
    }

    /// 检查是否为布尔值
    #[inline]
    pub fn is_boolean(&self) -> bool {
        matches!(self, XPathResult::Boolean(_))
    }

    /// 检查是否为数字
    #[inline]
    pub fn is_number(&self) -> bool {
        matches!(self, XPathResult::Number(_))
    }

    /// 检查是否为字符串
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, XPathResult::String(_))
    }

    /// 检查是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        matches!(self, XPathResult::Empty)
    }

    /// 获取节点集合，如果类型不匹配则返回空向量
    pub fn as_nodeset(&self) -> Vec<SelectedNode<'a>> {
        match self {
            XPathResult::NodeSet(nodes) => nodes.clone(),
            _ => Vec::new(),
        }
    }

    /// 获取布尔值，如果类型不匹配则进行转换
    pub fn as_boolean(&self) -> bool {
        match self {
            XPathResult::Boolean(b) => *b,
            XPathResult::Number(n) => *n != 0.0,
            XPathResult::NodeSet(nodes) => !nodes.is_empty(),
            XPathResult::String(s) => !s.is_empty(),
            XPathResult::Empty => false,
        }
    }

    /// 获取数字值，如果类型不匹配则进行转换
    pub fn as_number(&self) -> f64 {
        match self {
            XPathResult::Number(n) => *n,
            XPathResult::Boolean(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            XPathResult::String(s) => s.parse().unwrap_or(0.0),
            XPathResult::NodeSet(nodes) => nodes.len() as f64,
            XPathResult::Empty => 0.0,
        }
    }

    /// 获取字符串值，如果类型不匹配则进行转换
    pub fn as_string(&self) -> std::string::String {
        match self {
            XPathResult::String(s) => s.clone(),
            XPathResult::Number(n) => n.to_string(),
            XPathResult::Boolean(b) => b.to_string(),
            XPathResult::NodeSet(nodes) => {
                if nodes.is_empty() {
                    std::string::String::new()
                } else {
                    nodes[0].text()
                }
            }
            XPathResult::Empty => std::string::String::new(),
        }
    }
}

/// 执行 XPath 查询的内部实现
///
/// 此函数使用 libxml2-sys 提供的安全封装，对外暴露安全接口。
///
/// # Arguments
///
/// * `doc_ptr` - 有效的文档指针
/// * `xpath` - XPath 表达式字符串
pub(crate) fn evaluate_xpath<'a>(doc_ptr: xmlDocPtr, xpath: &str) -> Result<XPathResult<'a>> {
    let c_xpath = CString::new(xpath).map_err(|_| Error::NullByte)?;

    // SAFETY: doc_ptr 是有效的文档指针，c_xpath 是有效的 CString
    // 使用 libxml2-sys 提供的安全封装
    unsafe {
        let raw_result = xpath_evaluate(doc_ptr, c_xpath.as_ptr() as *const xmlChar)
            .ok_or_else(|| Error::invalid_xpath(xpath))?;

        let result = match raw_result.result_type {
            XPATH_NODESET => {
                let node_ptrs = raw_result.as_nodeset();
                let nodes = node_ptrs
                    .into_iter()
                    .map(|ptr| SelectedNode::from_raw(ptr))
                    .collect();
                XPathResult::NodeSet(nodes)
            }
            XPATH_BOOLEAN => XPathResult::Boolean(raw_result.as_boolean()),
            XPATH_NUMBER => XPathResult::Number(raw_result.as_number()),
            XPATH_STRING => XPathResult::String(raw_result.as_string()),
            _ => XPathResult::Empty,
        };

        Ok(result)
    }
}

/// 在节点上下文中执行 XPath 查询
///
/// # Arguments
///
/// * `node_ptr` - 有效的节点指针
/// * `xpath` - XPath 表达式字符串
pub(crate) fn evaluate_xpath_on_node<'a>(
    node_ptr: xmlNodePtr,
    xpath: &str,
) -> Result<Vec<SelectedNode<'a>>> {
    let c_xpath = CString::new(xpath).map_err(|_| Error::NullByte)?;

    // SAFETY: node_ptr 是有效的节点指针，c_xpath 是有效的 CString
    // 使用 libxml2-sys 提供的安全封装
    unsafe {
        let raw_result = xpath_evaluate_on_node(node_ptr, c_xpath.as_ptr() as *const xmlChar)
            .ok_or_else(|| Error::invalid_xpath(xpath))?;

        if raw_result.result_type == XPATH_NODESET {
            let node_ptrs = raw_result.as_nodeset();
            let nodes = node_ptrs
                .into_iter()
                .map(|ptr| SelectedNode::from_raw(ptr))
                .collect();
            Ok(nodes)
        } else {
            Ok(Vec::new())
        }
    }
}
