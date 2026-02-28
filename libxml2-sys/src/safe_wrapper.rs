//! libxml2 的安全封装函数
//!
//! 这个模块提供了对 libxml2 常用操作的安全封装，
//! 将 unsafe 操作集中在内部处理，对外暴露安全的 API。

use crate::{free_xml_char, xmlDocPtr, xmlNodePtr, xmlXPathContext, xmlXPathObject};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

// ========================================
// 字符串转换辅助函数
// ========================================

/// 将 C 字符串指针安全地转换为 Rust String
///
/// 如果指针为 null，返回空字符串。
///
/// # Safety
///
/// `ptr` 必须是有效的以 null 结尾的 C 字符串指针，或者为 null
#[inline]
pub unsafe fn ptr_to_string(ptr: *const i8) -> String {
    if ptr.is_null() {
        return String::new();
    }
    // SAFETY: 调用者保证 ptr 是有效的 C 字符串指针
    unsafe { CStr::from_ptr(ptr.cast()).to_string_lossy().into_owned() }
}

/// 将 C 字符串指针安全地转换为 Option<String>
///
/// 如果指针为 null，返回 None。
///
/// # Safety
///
/// `ptr` 必须是有效的以 null 结尾的 C 字符串指针，或者为 null
#[inline]
pub unsafe fn ptr_to_option_string(ptr: *const i8) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: 调用者保证 ptr 是有效的 C 字符串指针
    unsafe { Some(CStr::from_ptr(ptr.cast()).to_string_lossy().into_owned()) }
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
    // 注意：bindgen 在不同平台生成的 type_ 类型可能不同（u32 或 i32）
    unsafe { (*node).type_ as i32 }
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

// ========================================
// 文档解析封装
// ========================================

/// 从内存解析 HTML 文档
///
/// # Safety
///
/// - `html` 必须是有效的 C 字符串
/// - `size` 必须是 HTML 内容的实际长度
/// - `options` 必须是有效的解析选项组合
#[inline]
pub unsafe fn parse_html_memory(html: *const c_char, size: i32, options: i32) -> xmlDocPtr {
    // SAFETY: 调用者保证参数有效
    unsafe { crate::htmlReadMemory(html, size, ptr::null(), c"UTF-8".as_ptr(), options) }
}

/// 从内存解析 XML 文档
///
/// # Safety
///
/// - `xml` 必须是有效的 C 字符串
/// - `size` 必须是 XML 内容的实际长度
/// - `options` 必须是有效的解析选项组合
#[inline]
pub unsafe fn parse_xml_memory(xml: *const c_char, size: i32, options: i32) -> xmlDocPtr {
    // SAFETY: 调用者保证参数有效
    unsafe { crate::xmlReadMemory(xml, size, ptr::null(), ptr::null(), options) }
}

// ========================================
// XPath 操作封装
// ========================================

/// XPath 查询结果类型常量
pub const XPATH_NODESET: i32 = 1;
pub const XPATH_BOOLEAN: i32 = 2;
pub const XPATH_NUMBER: i32 = 3;
pub const XPATH_STRING: i32 = 4;

/// XPath 查询的原始结果
///
/// 用于内部传递 XPath 查询的原始结果，由调用者负责类型安全。
pub struct RawXPathResult {
    /// XPath 对象指针
    pub object: *mut xmlXPathObject,
    /// 结果类型
    pub result_type: i32,
}

impl RawXPathResult {
    /// 创建新的原始结果
    ///
    /// # Safety
    ///
    /// `object` 必须是有效的 xmlXPathObject 指针
    #[inline]
    pub unsafe fn new(object: *mut xmlXPathObject) -> Self {
        // SAFETY: 调用者保证 object 有效
        let result_type = unsafe { (*object).type_ as i32 };
        Self {
            object,
            result_type,
        }
    }

    /// 提取布尔值
    ///
    /// # Safety
    ///
    /// 必须是布尔类型的结果
    #[inline]
    pub unsafe fn as_boolean(&self) -> bool {
        // SAFETY: 调用者保证 object 有效且类型正确
        unsafe { (*self.object).boolval != 0 }
    }

    /// 提取数字值
    ///
    /// # Safety
    ///
    /// 必须是数字类型的结果
    #[inline]
    pub unsafe fn as_number(&self) -> f64 {
        // SAFETY: 调用者保证 object 有效且类型正确
        unsafe { (*self.object).floatval }
    }

    /// 提取字符串值
    ///
    /// # Safety
    ///
    /// 必须是字符串类型的结果
    #[inline]
    pub unsafe fn as_string(&self) -> String {
        // SAFETY: 调用者保证 object 有效且类型正确
        unsafe {
            let stringval = (*self.object).stringval;
            if stringval.is_null() {
                String::new()
            } else {
                ptr_to_string(stringval.cast())
            }
        }
    }

    /// 提取节点集合
    ///
    /// # Safety
    ///
    /// 必须是节点集合类型的结果
    #[inline]
    pub unsafe fn as_nodeset(&self) -> Vec<xmlNodePtr> {
        // SAFETY: 调用者保证 object 有效且类型正确
        unsafe {
            let mut nodes = Vec::new();
            let nodeset = (*self.object).nodesetval;
            if !nodeset.is_null() && (*nodeset).nodeNr > 0 {
                let node_count = (*nodeset).nodeNr as isize;
                let node_tab = (*nodeset).nodeTab;

                for i in 0..node_count {
                    let node = *node_tab.offset(i);
                    if !node.is_null() {
                        nodes.push(node);
                    }
                }
            }
            nodes
        }
    }
}

impl Drop for RawXPathResult {
    fn drop(&mut self) {
        // SAFETY: object 在 drop 时仍然有效
        unsafe {
            crate::xmlXPathFreeObject(self.object);
        }
    }
}

/// XPath 上下文守卫，确保正确释放资源
pub struct XPathContextGuard {
    ctx: *mut xmlXPathContext,
}

impl XPathContextGuard {
    /// 创建新的 XPath 上下文
    ///
    /// # Safety
    ///
    /// `doc` 必须是有效的文档指针
    #[inline]
    pub unsafe fn new(doc: xmlDocPtr) -> Option<Self> {
        // SAFETY: 调用者保证 doc 有效
        let ctx = unsafe { crate::xmlXPathNewContext(doc) };
        if ctx.is_null() {
            None
        } else {
            Some(Self { ctx })
        }
    }

    /// 设置上下文节点
    ///
    /// # Safety
    ///
    /// `node` 必须是有效的节点指针
    #[inline]
    pub unsafe fn set_context_node(&mut self, node: xmlNodePtr) -> bool {
        // SAFETY: 调用者保证 node 有效
        unsafe { crate::xmlXPathSetContextNode(node, self.ctx) == 0 }
    }

    /// 执行 XPath 表达式
    ///
    /// # Safety
    ///
    /// `xpath` 必须是有效的以 null 结尾的 C 字符串
    #[inline]
    pub unsafe fn evaluate(&mut self, xpath: *const crate::xmlChar) -> Option<RawXPathResult> {
        // SAFETY: 调用者保证参数有效
        let obj = unsafe { crate::xmlXPathEvalExpression(xpath, self.ctx) };
        if obj.is_null() {
            None
        } else {
            // SAFETY: obj 刚刚创建且非空
            Some(unsafe { RawXPathResult::new(obj) })
        }
    }
}

impl Drop for XPathContextGuard {
    fn drop(&mut self) {
        // SAFETY: ctx 在 drop 时仍然有效
        unsafe {
            crate::xmlXPathFreeContext(self.ctx);
        }
    }
}

/// 在文档上执行 XPath 查询
///
/// # Safety
///
/// - `doc` 必须是有效的文档指针
/// - `xpath` 必须是有效的以 null 结尾的 C 字符串
pub unsafe fn xpath_evaluate(
    doc: xmlDocPtr,
    xpath: *const crate::xmlChar,
) -> Option<RawXPathResult> {
    // SAFETY: 调用者保证参数有效
    let mut ctx = unsafe { XPathContextGuard::new(doc)? };
    unsafe { ctx.evaluate(xpath) }
}

/// 在节点上下文中执行 XPath 查询
///
/// # Safety
///
/// - `node` 必须是有效的节点指针
/// - `xpath` 必须是有效的以 null 结尾的 C 字符串
pub unsafe fn xpath_evaluate_on_node(
    node: xmlNodePtr,
    xpath: *const crate::xmlChar,
) -> Option<RawXPathResult> {
    // SAFETY: 调用者保证参数有效
    let doc = unsafe { node_get_document(node) };
    if doc.is_null() {
        return None;
    }

    let mut ctx = unsafe { XPathContextGuard::new(doc)? };
    if !unsafe { ctx.set_context_node(node) } {
        return None;
    }
    unsafe { ctx.evaluate(xpath) }
}

// ========================================
// 属性遍历高级封装
// ========================================

/// 获取节点的所有属性
///
/// 返回属性名和属性值的键值对向量。
///
/// # Safety
///
/// `node` 必须是有效的 xmlNodePtr
pub unsafe fn node_get_all_attributes(node: xmlNodePtr) -> Vec<(String, String)> {
    let mut result = Vec::new();

    // SAFETY: 调用者保证 node 有效
    let mut attr = unsafe { node_get_first_attribute(node) };
    while !attr.is_null() {
        let name_ptr = unsafe { attr_get_name(attr) };
        if !name_ptr.is_null() {
            let name = unsafe { ptr_to_string(name_ptr.cast()) };

            // 获取属性值
            let value_ptr = unsafe { crate::xmlGetProp(node, name_ptr) };
            if !value_ptr.is_null() {
                let value = unsafe { ptr_to_string(value_ptr.cast()) };
                result.push((name, value));
                unsafe { free_xml_char(value_ptr) };
            }
        }
        attr = unsafe { attr_get_next(attr) };
    }

    result
}

/// 检查节点是否具有指定属性
///
/// # Safety
///
/// - `node` 必须是有效的 xmlNodePtr
/// - `name` 必须是有效的以 null 结尾的 C 字符串
pub unsafe fn node_has_attribute(node: xmlNodePtr, name: *const crate::xmlChar) -> bool {
    // SAFETY: 调用者保证参数有效
    let prop = unsafe { crate::xmlGetProp(node, name) };
    if prop.is_null() {
        return false;
    }
    unsafe { free_xml_char(prop) };
    true
}
