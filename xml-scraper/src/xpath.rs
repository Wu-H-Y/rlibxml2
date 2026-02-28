//! XPath 查询与结果处理

use crate::error::{Error, Result};
use crate::node::SelectedNode;
use libxml2_sys::*;
use std::ffi::{CStr, CString};

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
    String(String),
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
    pub fn as_string(&self) -> String {
        match self {
            XPathResult::String(s) => s.clone(),
            XPathResult::Number(n) => n.to_string(),
            XPathResult::Boolean(b) => b.to_string(),
            XPathResult::NodeSet(nodes) => {
                if nodes.is_empty() {
                    String::new()
                } else {
                    nodes[0].text()
                }
            }
            XPathResult::Empty => String::new(),
        }
    }
}

/// XPath 查询类型常量
pub(crate) const XPATH_NODESET: i32 = 1;
pub(crate) const XPATH_BOOLEAN: i32 = 2;
pub(crate) const XPATH_NUMBER: i32 = 3;
pub(crate) const XPATH_STRING: i32 = 4;

// ========================================
// 私有 unsafe helper 函数
// ========================================

/// 将 C 字符串指针安全地转换为 String
///
/// # Safety
///
/// `ptr` 必须是有效的以 null 结尾的 C 字符串指针，或者为 null
#[inline]
unsafe fn ptr_to_string(ptr: *const i8) -> String {
    if ptr.is_null() {
        return String::new();
    }
    // SAFETY: 调用者保证 ptr 是有效的 C 字符串
    unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

/// 创建 XPath 上下文并执行查询
///
/// # Safety
///
/// - `doc_ptr` 必须是有效的文档指针
/// - `c_xpath` 必须是有效的以 null 结尾的 C 字符串
unsafe fn create_xpath_context(
    doc_ptr: xmlDocPtr,
    c_xpath: &CString,
) -> Result<(*mut xmlXPathContext, *mut xmlXPathObject)> {
    // SAFETY: 调用者保证 doc_ptr 是有效的
    let ctx = unsafe { xmlXPathNewContext(doc_ptr) };
    if ctx.is_null() {
        return Err(Error::XPathContextFailed);
    }

    // SAFETY: c_xpath 是有效的 CString
    let xpath_obj = unsafe { xmlXPathEvalExpression(c_xpath.as_ptr() as *const xmlChar, ctx) };

    if xpath_obj.is_null() {
        unsafe { xmlXPathFreeContext(ctx) };
        return Err(Error::invalid_xpath(c_xpath.to_str().unwrap_or("")));
    }

    Ok((ctx, xpath_obj))
}

/// 从 XPath 结果对象中提取节点集合
///
/// # Safety
///
/// `xpath_obj` 必须是有效的 XPath 对象指针，且类型为 XPATH_NODESET
unsafe fn extract_nodeset(xpath_obj: *mut xmlXPathObject) -> Vec<SelectedNode<'static>> {
    let mut nodes = Vec::new();

    // SAFETY: 调用者保证 xpath_obj 是有效的
    let nodeset = unsafe { (*xpath_obj).nodesetval };
    if !nodeset.is_null() && unsafe { (*nodeset).nodeNr } > 0 {
        let node_count = unsafe { (*nodeset).nodeNr } as isize;
        let node_tab = unsafe { (*nodeset).nodeTab };

        for i in 0..node_count {
            // SAFETY: node_tab 是有效的数组，node_count 是其长度
            let node = unsafe { *node_tab.offset(i) };
            if !node.is_null() {
                // SAFETY: node 是有效的节点指针
                nodes.push(unsafe { SelectedNode::from_raw(node) });
            }
        }
    }

    nodes
}

/// 执行 XPath 查询的内部实现
///
/// 此函数封装了所有 unsafe 操作，对外暴露安全接口。
///
/// # Safety
///
/// - `doc_ptr` 必须是有效的文档指针
pub(crate) fn evaluate_xpath<'a>(doc_ptr: xmlDocPtr, xpath: &str) -> Result<XPathResult<'a>> {
    let c_xpath = CString::new(xpath).map_err(|_| Error::NullByte)?;

    // SAFETY: doc_ptr 必须是有效的文档指针，c_xpath 是有效的 CString
    unsafe {
        let (ctx, xpath_obj) = create_xpath_context(doc_ptr, &c_xpath)?;

        let result_type = (*xpath_obj).type_;
        let result = match result_type {
            XPATH_NODESET => {
                let nodes = extract_nodeset(xpath_obj);
                XPathResult::NodeSet(nodes)
            }
            XPATH_BOOLEAN => XPathResult::Boolean((*xpath_obj).boolval != 0),
            XPATH_NUMBER => XPathResult::Number((*xpath_obj).floatval),
            XPATH_STRING => {
                let stringval = (*xpath_obj).stringval;
                if stringval.is_null() {
                    XPathResult::String(String::new())
                } else {
                    XPathResult::String(ptr_to_string(stringval.cast()))
                }
            }
            _ => XPathResult::Empty,
        };

        xmlXPathFreeObject(xpath_obj);
        xmlXPathFreeContext(ctx);

        Ok(result)
    }
}
