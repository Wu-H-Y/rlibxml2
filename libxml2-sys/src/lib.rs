//! libxml2-sys - libxml2 的原始 FFI 绑定
//!
//! 这个 crate 提供了对 libxml2 库的 unsafe 原始绑定。
//! 如果你想使用安全的 API，请使用 `xml-scraper` crate。

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// 安全封装模块
mod safe_wrapper;
pub use safe_wrapper::*;

// ========================================
// 指针类型别名
// ========================================

/// XML 文档指针
pub type xmlDocPtr = *mut xmlDoc;

/// XML 节点指针
pub type xmlNodePtr = *mut xmlNode;

/// XML 属性指针
pub type xmlAttrPtr = *mut xmlAttr;

/// XML 命名空间指针
pub type xmlNsPtr = *mut xmlNs;

/// XPath 上下文指针
pub type xmlXPathContextPtr = *mut xmlXPathContext;

// 注意: xmlXPathObjectPtr 和 xmlNodeSetPtr 已由 bindgen 自动生成

/// XPath 解析上下文指针
pub type xmlXPathParserContextPtr = *mut xmlXPathParserContext;

/// XPath 编译表达式指针
pub type xmlXPathCompExprPtr = *mut xmlXPathCompExpr;

/// XPath 类型指针
pub type xmlXPathTypePtr = *mut xmlXPathType;

/// XPath 变量指针
pub type xmlXPathVariablePtr = *mut xmlXPathVariable;

/// XPath 函数指针
pub type xmlXPathFuncPtr = *mut xmlXPathFunct;

/// XPath 轴指针
pub type xmlXPathAxisPtr = *mut xmlXPathAxis;

/// XML 字典指针
pub type xmlDictPtr = *mut xmlDict;

/// XML 错误指针
pub type xmlErrorPtr = *mut xmlError;

/// XML DTD 指针
pub type xmlDtdPtr = *mut xmlDtd;

/// XML 哈希表指针
pub type xmlHashTablePtr = *mut xmlHashTable;

// ========================================
// 辅助函数
// ========================================

/// 安全释放 libxml2 分配的 xmlChar* 字符串
///
/// # Safety
///
/// - `ptr` 必须是由 libxml2 分配的有效指针，或者为 null
/// - 释放后不能再使用该指针
#[inline]
pub unsafe fn free_xml_char(ptr: *mut xmlChar) {
    // SAFETY: 调用者保证 ptr 是有效的 libxml2 分配的指针或 null
    unsafe {
        if !ptr.is_null()
            && let Some(free_fn) = xmlFree
        {
            free_fn(ptr as *mut std::ffi::c_void);
        }
    }
}

/// 初始化 XML 解析器
///
/// 在多线程环境中使用 libxml2 时，应该在程序开始时调用此函数。
/// 这是 `xmlInitParser()` 的安全包装。
pub fn init_parser() {
    unsafe {
        xmlInitParser();
    }
}

/// 清理 XML 解析器
///
/// 当程序不再需要解析 HTML/XML 时调用此函数来释放资源。
/// 这是 `xmlCleanupParser()` 的安全包装。
pub fn cleanup_parser() {
    unsafe {
        xmlCleanupParser();
    }
}
