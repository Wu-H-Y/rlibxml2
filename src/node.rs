//! 节点表示与操作
//!
//! 提供对 XML/HTML DOM 节点的安全访问。

use crate::error::Result;
use crate::node_type::NodeType;
use crate::xpath::evaluate_xpath_on_node;
use libxml2_sys::*;
use std::collections::HashMap;
use std::ffi::CString;
use std::marker::PhantomData;

/// 带生命周期绑定的安全节点引用
///
/// `SelectedNode<'a>` 的生命周期绑定到所属的 `Document`，
/// 确保节点不会在文档被释放后继续使用。
///
/// # 生命周期安全
///
/// ```compile_fail
/// use rlibxml::Document;
///
/// let node = {
///     let doc = Document::parse("<div>test</div>").unwrap();
///     doc.select("//div").unwrap()[0].clone()
///     // doc 在此处被 drop
/// };
/// node.text();  // 编译错误：`doc` 的生命周期不够长
/// ```
///
/// 正确的用法是确保文档在节点使用期间保持存活：
///
/// ```
/// use rlibxml::Document;
///
/// let doc = Document::parse("<div>test</div>").unwrap();
/// let node = &doc.select("//div").unwrap()[0];
/// println!("{}", node.text());  // OK: doc 仍然存活
/// ```
#[derive(Debug)]
pub struct SelectedNode<'a> {
    pub(crate) node_ptr: xmlNodePtr,
    pub(crate) _marker: PhantomData<&'a ()>,
}

// SAFETY: SelectedNode 只是不可变的借用引用，底层的 xmlNodePtr 的实际所有者是线程安全的 Document。
// 只要 Document 依然存活，跨线程传递借用是安全的。
unsafe impl<'a> Send for SelectedNode<'a> {}
unsafe impl<'a> Sync for SelectedNode<'a> {}

// 手动实现 Clone，不要求 Document: Clone
impl<'a> Clone for SelectedNode<'a> {
    fn clone(&self) -> Self {
        Self {
            node_ptr: self.node_ptr,
            _marker: PhantomData,
        }
    }
}

impl<'a> SelectedNode<'a> {
    /// 创建新的节点引用
    ///
    /// # Safety
    ///
    /// - `node_ptr` 必须是有效的 libxml2 节点指针
    /// - 节点必须在 `'a` 生命周期内保持有效
    #[inline]
    pub(crate) unsafe fn from_raw(node_ptr: xmlNodePtr) -> Self {
        Self {
            node_ptr,
            _marker: PhantomData,
        }
    }

    // ========================================
    // 私有 unsafe helper 方法
    // ========================================

    /// 从原始指针创建节点，如果为 null 则返回 None
    #[inline]
    unsafe fn from_raw_option(ptr: xmlNodePtr) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            // SAFETY: ptr 已经检查非空
            Some(unsafe { Self::from_raw(ptr) })
        }
    }

    // ========================================
    // 公开安全 API
    // ========================================

    /// 获取节点的文本内容
    ///
    /// 对于文本节点，返回文本内容；对于元素节点，返回所有子文本的连接。
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let doc = Document::parse("<div>Hello <span>World</span></div>").unwrap();
    /// let node = &doc.select("//div").unwrap()[0];
    /// assert_eq!(node.text(), "Hello World");
    /// ```
    pub fn text(&self) -> String {
        // SAFETY: node_ptr 在节点存活期间始终有效
        // 使用 libxml2-sys 提供的安全封装函数
        unsafe { node_get_content(self.node_ptr) }
    }

    /// 获取节点的标签名
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let doc = Document::parse("<div class='container'>Hello</div>").unwrap();
    /// let node = &doc.select("//div").unwrap()[0];
    /// assert_eq!(node.tag_name(), "div");
    /// ```
    pub fn tag_name(&self) -> String {
        // SAFETY: node_ptr 在节点存活期间始终有效
        // 使用 libxml2-sys 提供的安全封装函数
        unsafe { node_get_name(self.node_ptr) }
    }

    /// 获取节点的 XPath 路径
    ///
    /// 返回从文档根节点到当前节点的绝对路径。
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let doc = Document::parse("<html><body><div><p>text</p></div></body></html>").unwrap();
    /// let node = &doc.select("//p").unwrap()[0];
    /// assert!(node.path().contains("/p"));
    /// ```
    pub fn path(&self) -> String {
        // SAFETY: node_ptr 在节点存活期间始终有效
        // 使用 libxml2-sys 提供的安全封装函数
        unsafe { node_get_path(self.node_ptr) }
    }

    /// 获取节点的类型
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::{Document, NodeType};
    ///
    /// let doc = Document::parse("<p>Text<!-- comment --></p>").unwrap();
    /// let elements = doc.select("//p").unwrap();
    /// assert_eq!(elements[0].node_type(), NodeType::Element);
    ///
    /// let text_nodes = doc.select("//p/text()").unwrap();
    /// assert_eq!(text_nodes[0].node_type(), NodeType::Text);
    /// ```
    pub fn node_type(&self) -> NodeType {
        // SAFETY: node_ptr 在节点存活期间始终有效
        // 使用 libxml2-sys 提供的安全封装函数
        let type_val = unsafe { node_get_type(self.node_ptr) };
        NodeType::from_raw(type_val)
    }

    // ========================================
    // 属性访问 API
    // ========================================

    /// 获取指定属性的值
    ///
    /// 如果属性不存在，返回 `None`。
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let doc = Document::parse("<div class='container' id='main'>Hello</div>").unwrap();
    /// let node = &doc.select("//div").unwrap()[0];
    /// assert_eq!(node.attr("class"), Some("container".to_string()));
    /// assert_eq!(node.attr("id"), Some("main".to_string()));
    /// assert_eq!(node.attr("style"), None);
    /// ```
    pub fn attr(&self, name: &str) -> Option<String> {
        if name.is_empty() {
            return None;
        }

        let c_name = CString::new(name).ok()?;
        // SAFETY: node_ptr 和 c_name 都有效
        // 使用 libxml2-sys 提供的安全封装函数
        unsafe { node_get_attribute(self.node_ptr, c_name.as_ptr().cast()) }
    }

    /// 获取所有属性
    ///
    /// 返回属性名到属性值的映射。
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let doc = Document::parse("<div class='container' id='main'>Hello</div>").unwrap();
    /// let node = &doc.select("//div").unwrap()[0];
    /// let attrs = node.attrs();
    /// assert_eq!(attrs.get("class"), Some(&"container".to_string()));
    /// assert_eq!(attrs.get("id"), Some(&"main".to_string()));
    /// ```
    pub fn attrs(&self) -> HashMap<String, String> {
        // SAFETY: node_ptr 在节点存活期间始终有效
        // 使用 libxml2-sys 提供的安全封装函数
        let attrs_vec = unsafe { node_get_all_attributes(self.node_ptr) };
        attrs_vec.into_iter().collect()
    }

    /// 检查是否具有指定属性
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let doc = Document::parse("<div class='container'>Hello</div>").unwrap();
    /// let node = &doc.select("//div").unwrap()[0];
    /// assert!(node.has_attr("class"));
    /// assert!(!node.has_attr("style"));
    /// ```
    pub fn has_attr(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        let c_name = match CString::new(name) {
            Ok(c) => c,
            Err(_) => return false, // 包含空字节
        };

        // SAFETY: node_ptr 和 c_name 都有效
        // 使用 libxml2-sys 提供的安全封装函数
        unsafe { node_has_attribute(self.node_ptr, c_name.as_ptr().cast()) }
    }

    // ========================================
    // 节点遍历 API
    // ========================================

    /// 获取第一个子节点
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let doc = Document::parse("<div><p>text</p><span>more</span></div>").unwrap();
    /// let div = &doc.select("//div").unwrap()[0];
    /// let first_child = div.first_child().unwrap();
    /// assert_eq!(first_child.tag_name(), "p");
    /// ```
    pub fn first_child(&self) -> Option<SelectedNode<'a>> {
        // SAFETY: node_ptr 在节点存活期间始终有效
        // 使用 libxml2-sys 提供的安全封装函数
        unsafe { Self::from_raw_option(node_get_first_child(self.node_ptr)) }
    }

    /// 获取最后一个子节点
    pub fn last_child(&self) -> Option<SelectedNode<'a>> {
        // SAFETY: node_ptr 在节点存活期间始终有效
        // 使用 libxml2-sys 提供的安全封装函数
        unsafe { Self::from_raw_option(node_get_last_child(self.node_ptr)) }
    }

    /// 获取所有子节点
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let doc = Document::parse("<div><p>A</p><p>B</p></div>").unwrap();
    /// let div = &doc.select("//div").unwrap()[0];
    /// let children = div.children();
    /// assert_eq!(children.len(), 2);
    /// ```
    pub fn children(&self) -> Vec<SelectedNode<'a>> {
        let mut result = Vec::new();

        // SAFETY: node_ptr 在节点存活期间始终有效
        // 使用 libxml2-sys 提供的安全封装函数
        unsafe {
            let mut child = node_get_first_child(self.node_ptr);
            while !child.is_null() {
                result.push(Self::from_raw(child));
                child = node_get_next_sibling(child);
            }
        }

        result
    }

    /// 获取元素子节点（仅元素，不包括文本节点）
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let doc = Document::parse("<div>text<p>A</p>more<p>B</p></div>").unwrap();
    /// let div = &doc.select("//div").unwrap()[0];
    /// let elements = div.element_children();
    /// assert_eq!(elements.len(), 2);
    /// ```
    pub fn element_children(&self) -> Vec<SelectedNode<'a>> {
        self.children()
            .into_iter()
            .filter(|n| n.node_type().is_element())
            .collect()
    }

    /// 获取文本子节点内容
    pub fn text_children(&self) -> Vec<String> {
        self.children()
            .into_iter()
            .filter(|n| n.node_type().is_text())
            .map(|n| n.text())
            .collect()
    }

    /// 获取父节点
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let doc = Document::parse("<div><p>text</p></div>").unwrap();
    /// let p = &doc.select("//p").unwrap()[0];
    /// let parent = p.parent().unwrap();
    /// assert_eq!(parent.tag_name(), "div");
    /// ```
    pub fn parent(&self) -> Option<SelectedNode<'a>> {
        // SAFETY: node_ptr 在节点存活期间始终有效
        // 使用 libxml2-sys 提供的安全封装函数
        unsafe { Self::from_raw_option(node_get_parent(self.node_ptr)) }
    }

    /// 获取下一个兄弟节点
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let doc = Document::parse("<div><p id='a'>A</p><p id='b'>B</p></div>").unwrap();
    /// let first_p = &doc.select("//p[@id='a']").unwrap()[0];
    /// let next = first_p.next_sibling().unwrap();
    /// assert_eq!(next.attr("id"), Some("b".to_string()));
    /// ```
    pub fn next_sibling(&self) -> Option<SelectedNode<'a>> {
        // SAFETY: node_ptr 在节点存活期间始终有效
        // 使用 libxml2-sys 提供的安全封装函数
        unsafe { Self::from_raw_option(node_get_next_sibling(self.node_ptr)) }
    }

    /// 获取上一个兄弟节点
    pub fn prev_sibling(&self) -> Option<SelectedNode<'a>> {
        // SAFETY: node_ptr 在节点存活期间始终有效
        // 使用 libxml2-sys 提供的安全封装函数
        unsafe { Self::from_raw_option(node_get_prev_sibling(self.node_ptr)) }
    }

    /// 获取所有兄弟节点（不包括自身）
    pub fn siblings(&self) -> Vec<SelectedNode<'a>> {
        let mut result = Vec::new();

        // SAFETY: node_ptr 在节点存活期间始终有效
        // 使用 libxml2-sys 提供的安全封装函数
        // 向前遍历
        unsafe {
            let mut prev = node_get_prev_sibling(self.node_ptr);
            while !prev.is_null() {
                result.push(Self::from_raw(prev));
                prev = node_get_prev_sibling(prev);
            }
        }

        result.reverse();

        // 向后遍历
        unsafe {
            let mut next = node_get_next_sibling(self.node_ptr);
            while !next.is_null() {
                result.push(Self::from_raw(next));
                next = node_get_next_sibling(next);
            }
        }

        result
    }

    /// 检查节点是否有子节点
    pub fn has_children(&self) -> bool {
        // SAFETY: node_ptr 在节点存活期间始终有效
        // 使用 libxml2-sys 提供的安全封装函数
        unsafe { node_has_children(self.node_ptr) }
    }

    /// 检查节点是否有父节点
    pub fn has_parent(&self) -> bool {
        // SAFETY: node_ptr 在节点存活期间始终有效
        // 使用 libxml2-sys 提供的安全封装函数
        unsafe { node_has_parent(self.node_ptr) }
    }

    /// 获取子节点数量
    pub fn child_count(&self) -> usize {
        self.children().len()
    }

    /// 获取节点的内部 HTML（序列化为字符串）
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let doc = Document::parse("<div><p>Hello</p><span>World</span></div>").unwrap();
    /// let div = &doc.select("//div").unwrap()[0];
    /// let html = div.inner_html();
    /// assert!(html.contains("<p>"));
    /// assert!(html.contains("<span>"));
    /// ```
    pub fn inner_html(&self) -> String {
        let children = self.children();
        children.iter().map(|c| c.outer_html()).collect()
    }

    /// 获取节点的外部 HTML（包含节点自身）
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let doc = Document::parse("<div><p>Hello</p></div>").unwrap();
    /// let p = &doc.select("//p").unwrap()[0];
    /// let html = p.outer_html();
    /// assert!(html.starts_with("<p>"));
    /// ```
    pub fn outer_html(&self) -> String {
        // 简化实现：拼接标签和内容
        let tag = self.tag_name();
        if tag.is_empty() {
            return self.text();
        }

        let attrs = self.attrs();
        let attr_str = if attrs.is_empty() {
            String::new()
        } else {
            let pairs: Vec<String> = attrs
                .iter()
                .map(|(k, v)| format!("{}=\"{}\"", k, v.replace('"', "&quot;")))
                .collect();
            format!(" {}", pairs.join(" "))
        };

        let inner = self.inner_html();
        format!("<{}{}>{}</{}>", tag, attr_str, inner, tag)
    }

    /// 在当前节点上下文中执行 XPath 查询
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let doc = Document::parse("<div><p class='a'>A</p><p class='b'>B</p></div>").unwrap();
    /// let div = &doc.select("//div").unwrap()[0];
    /// let paragraphs = div.select(".//p").unwrap();
    /// assert_eq!(paragraphs.len(), 2);
    /// ```
    pub fn select(&self, xpath: &str) -> Result<Vec<SelectedNode<'a>>> {
        // 使用 libxml2-sys 提供的安全封装
        evaluate_xpath_on_node(self.node_ptr, xpath)
    }

    /// 获取原始节点指针（用于高级用途）
    ///
    /// # Safety
    ///
    /// 调用者必须确保：
    /// - 在使用返回的指针期间，文档仍然存活
    /// - 不通过此指针释放或修改节点
    #[inline]
    pub unsafe fn as_ptr(&self) -> xmlNodePtr {
        self.node_ptr
    }
}

impl<'a> std::fmt::Display for SelectedNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node({} at {})", self.tag_name(), self.path())
    }
}
