//! libxml2 的安全封装函数
//!
//! 这个模块提供了对 libxml2 常用操作的安全封装，
//! 将 unsafe 操作集中在内部处理，对外暴露安全的 API。

use crate::{free_xml_char, xmlDocPtr, xmlNodePtr};
use std::ffi::CStr;

// ========================================
// 字符串转换辅助函数
// ========================================

/// 将 C 字符串指针安全地转换为 Rust String
///
/// 如果指针为 null，返回空字符串。
#[inline]
pub(crate) unsafe fn ptr_to_string(ptr: *const i8) -> String {
    if ptr.is_null() {
        return String::new();
    }
    // SAFETY: 调用者保证 ptr 是有效的 C 字符串指针
    unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

/// 将 C 字符串指针安全地转换为 Option<String>
///
/// 如果指针为 null，返回 None。
#[inline]
pub(crate) unsafe fn ptr_to_option_string(ptr: *const i8) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: 调用者保证 ptr 是有效的 C 字符串指针
    unsafe { Some(CStr::from_ptr(ptr).to_string_lossy().into_owned()) }
}

// ========================================
// 节点操作封装
// ========================================

/// 获取节点的文本内容
///
/// # Safety
///
/// `node` 必须是有效的 xmlNodePtr
#[inline]
pub unsafe fn node_get_content(node: xmlNodePtr) -> String {
    // SAFETY: 调用者保证 node 是有效的
    let text_ptr = unsafe { crate::xmlNodeGetContent(node) };
    if text_ptr.is_null() {
        return String::new();
    }

    let result = unsafe { ptr_to_string(text_ptr.cast()) };
    unsafe { free_xml_char(text_ptr.cast()) };
    result
}

/// 获取节点的标签名
///
/// # Safety
///
/// `node` 必须是有效的 xmlNodePtr
#[inline]
pub unsafe fn node_get_name(node: xmlNodePtr) -> String {
    // SAFETY: 调用者保证 node 是有效的
    unsafe {
        if (*node).name.is_null() {
            return String::new();
        }
        ptr_to_string((*node).name.cast())
    }
}

/// 获取节点的 XPath 路径
///
/// # Safety
///
/// `node` 必须是有效的 xmlNodePtr
#[inline]
pub unsafe fn node_get_path(node: xmlNodePtr) -> String {
    // SAFETY: 调用者保证 node 是有效的
    let path_ptr = unsafe { crate::xmlGetNodePath(node) };
    if path_ptr.is_null() {
        return String::new();
    }
    let result = unsafe { ptr_to_string(path_ptr.cast()) };
    unsafe { free_xml_char(path_ptr.cast()) };
    result
}

/// 获取节点的类型
///
/// # Safety
///
/// `node` 必须是有效的 xmlNodePtr
#[inline]
pub unsafe fn node_get_type(node: xmlNodePtr) -> i32 {
    // SAFETY: 调用者保证 node 是有效的
    unsafe { (*node).type_ }
}

/// 获取节点的属性值
///
/// # Safety
///
/// - `node` 必须是有效的 xmlNodePtr
/// - `name` 必须是有效的以 null 结尾的 C 字符串
#[inline]
pub unsafe fn node_get_attribute(node: xmlNodePtr, name: *const crate::xmlChar) -> Option<String> {
    // SAFETY: 调用者保证参数有效
    let prop = unsafe { crate::xmlGetProp(node, name) };
    if prop.is_null() {
        return None;
    }
    let result = unsafe { ptr_to_option_string(prop.cast()) };
    unsafe { free_xml_char(prop) };
    result
}

/// 获取文档的根节点
///
/// # Safety
///
/// `doc` 必须是有效的 xmlDocPtr
#[inline]
pub unsafe fn doc_get_root_element(doc: xmlDocPtr) -> xmlNodePtr {
    // SAFETY: 调用者保证 doc 是有效的
    unsafe { crate::xmlDocGetRootElement(doc) }
}

/// 释放文档
///
/// # Safety
///
/// `doc` 必须是有效的 xmlDocPtr，且只能释放一次
#[inline]
pub unsafe fn doc_free(doc: xmlDocPtr) {
    // SAFETY: 调用者保证 doc 是有效的且只释放一次
    if !doc.is_null() {
        unsafe { crate::xmlFreeDoc(doc) };
    }
}

// ========================================
// 节点遍历封装
// ========================================

/// 获取节点的第一个子节点
///
/// # Safety
///
/// `node` 必须是有效的 xmlNodePtr
#[inline]
pub unsafe fn node_get_first_child(node: xmlNodePtr) -> xmlNodePtr {
    // SAFETY: 调用者保证 node 是有效的
    unsafe { (*node).children }
}

/// 获取节点的最后一个子节点
///
/// # Safety
///
/// `node` 必须是有效的 xmlNodePtr
#[inline]
pub unsafe fn node_get_last_child(node: xmlNodePtr) -> xmlNodePtr {
    // SAFETY: 调用者保证 node 是有效的
    unsafe { (*node).last }
}

/// 获取节点的下一个兄弟节点
///
/// # Safety
///
/// `node` 必须是有效的 xmlNodePtr
#[inline]
pub unsafe fn node_get_next_sibling(node: xmlNodePtr) -> xmlNodePtr {
    // SAFETY: 调用者保证 node 是有效的
    unsafe { (*node).next }
}

/// 获取节点的上一个兄弟节点
///
/// # Safety
///
/// `node` 必须是有效的 xmlNodePtr
#[inline]
pub unsafe fn node_get_prev_sibling(node: xmlNodePtr) -> xmlNodePtr {
    // SAFETY: 调用者保证 node 是有效的
    unsafe { (*node).prev }
}

/// 获取节点的父节点
///
/// # Safety
///
/// `node` 必须是有效的 xmlNodePtr
#[inline]
pub unsafe fn node_get_parent(node: xmlNodePtr) -> xmlNodePtr {
    // SAFETY: 调用者保证 node 是有效的
    unsafe { (*node).parent }
}

/// 获取节点所属的文档
///
/// # Safety
///
/// `node` 必须是有效的 xmlNodePtr
#[inline]
pub unsafe fn node_get_document(node: xmlNodePtr) -> xmlDocPtr {
    // SAFETY: 调用者保证 node 是有效的
    unsafe { (*node).doc }
}

/// 检查节点是否有子节点
///
/// # Safety
///
/// `node` 必须是有效的 xmlNodePtr
#[inline]
pub unsafe fn node_has_children(node: xmlNodePtr) -> bool {
    // SAFETY: 调用者保证 node 是有效的
    unsafe { !(*node).children.is_null() }
}

/// 检查节点是否有父节点
///
/// # Safety
///
/// `node` 必须是有效的 xmlNodePtr
#[inline]
pub unsafe fn node_has_parent(node: xmlNodePtr) -> bool {
    // SAFETY: 调用者保证 node 是有效的
    unsafe { !(*node).parent.is_null() }
}

// ========================================
// 属性遍历封装
// ========================================

use crate::xmlAttr;

/// 获取节点的第一个属性
///
/// # Safety
///
/// `node` 必须是有效的 xmlNodePtr
#[inline]
pub unsafe fn node_get_first_attribute(node: xmlNodePtr) -> *mut xmlAttr {
    // SAFETY: 调用者保证 node 是有效的
    unsafe { (*node).properties }
}

/// 获取下一个属性
///
/// # Safety
///
/// `attr` 必须是有效的 xmlAttrPtr
#[inline]
pub unsafe fn attr_get_next(attr: *mut xmlAttr) -> *mut xmlAttr {
    // SAFETY: 调用者保证 attr 是有效的
    unsafe { (*attr).next }
}

/// 获取属性的名称
///
/// # Safety
///
/// `attr` 必须是有效的 xmlAttrPtr
#[inline]
pub unsafe fn attr_get_name(attr: *mut xmlAttr) -> *const crate::xmlChar {
    // SAFETY: 调用者保证 attr 是有效的
    unsafe { (*attr).name }
}
