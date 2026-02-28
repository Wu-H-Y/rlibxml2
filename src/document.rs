//! 文档解析与管理
//!
//! 提供对 XML/HTML 文档的解析和生命周期管理。

use crate::error::{Error, Result};
use crate::node::SelectedNode;
use crate::options::{ParseOptions, XmlParseOptions};
use crate::xpath::{XPathResult, evaluate_xpath};
use libxml2_sys::*;
use std::ffi::CString;
use std::marker::PhantomData;
use std::ptr;

/// 输入数据的最大大小（略小于 2GB 以留出安全边界）
pub(crate) const MAX_INPUT_SIZE: usize = i32::MAX as usize - 1024;

/// 解析后的 XML/HTML 文档
///
/// 这是一个 RAII 类型，当它被 drop 时会自动释放整个 DOM 树。
/// 所有从此文档创建的节点引用（`SelectedNode`）的生命周期都绑定到此文档。
///
/// # 生命周期安全
///
/// 文档被 drop 后，所有从该文档创建的节点引用将变为无效。
/// Rust 的生命周期系统会在编译时阻止这种情况：
///
/// ```compile_fail
/// use rlibxml::Document;
///
/// let node = {
///     let doc = Document::parse("<div>test</div>").unwrap();
///     doc.select("//div").unwrap()[0].clone()
///     // doc 在此处被 drop
/// };
/// node.text();  // 编译错误！
/// ```
///
/// # Example
///
/// ```
/// use rlibxml::Document;
///
/// // 解析 HTML（默认容错模式）
/// let doc = Document::parse("<div>Hello</div>").unwrap();
///
/// // 解析 XML（严格模式）
/// let doc = Document::parse_xml("<root><item>data</item></root>").unwrap();
///
/// // 使用自定义选项解析 HTML
/// use rlibxml::ParseOptions;
/// let html = "<div>Hello</div>";
/// let doc = Document::parse_html_with_options(html, ParseOptions::default()).unwrap();
/// ```
pub struct Document {
    doc_ptr: xmlDocPtr,
    // 防止跨线程发送（裸指针使得类型 !Send + !Sync）
    _marker: PhantomData<*const ()>,
}

impl Document {
    /// 从 HTML 字符串解析文档
    ///
    /// 使用最大容错模式解析，适合处理真实世界的脏 HTML。
    ///
    /// # Arguments
    ///
    /// * `html` - HTML 字符串（必须是有效的 UTF-8）
    ///
    /// # Errors
    ///
    /// - [`Error::NullByte`] - HTML 包含空字节
    /// - [`Error::InputTooLarge`] - 输入数据超过大小限制
    /// - [`Error::ParseFailed`] - 解析失败
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let doc = Document::parse("<div>Hello</div>").unwrap();
    /// ```
    pub fn parse(html: &str) -> Result<Self> {
        Self::parse_html_with_options(html, ParseOptions::default())
    }

    /// 从 HTML 字符串解析文档（显式方法）
    ///
    /// 等价于 [`Document::parse`]，用于代码可读性。
    pub fn parse_html(html: &str) -> Result<Self> {
        Self::parse(html)
    }

    /// 使用自定义选项解析 HTML 文档
    ///
    /// # Arguments
    ///
    /// * `html` - HTML 字符串
    /// * `options` - 解析选项
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::{Document, ParseOptions};
    ///
    /// let html = "<div>Hello</div>";
    /// let options = ParseOptions {
    ///     recover: true,
    ///     no_error: true,
    ///     no_warning: true,
    ///     no_blanks: true,
    /// };
    /// let doc = Document::parse_html_with_options(html, options).unwrap();
    /// ```
    pub fn parse_html_with_options(html: &str, options: ParseOptions) -> Result<Self> {
        // 检查输入大小
        let size = html.len();
        if size > MAX_INPUT_SIZE {
            return Err(Error::InputTooLarge {
                size,
                max: MAX_INPUT_SIZE,
            });
        }

        let c_html = CString::new(html).map_err(|_| Error::NullByte)?;

        // 构建 HTML 解析选项
        let mut raw_options: htmlParserOption = 0;
        if options.recover {
            raw_options |= htmlParserOption_HTML_PARSE_RECOVER;
        }
        if options.no_error {
            raw_options |= htmlParserOption_HTML_PARSE_NOERROR;
        }
        if options.no_warning {
            raw_options |= htmlParserOption_HTML_PARSE_NOWARNING;
        }
        if options.no_blanks {
            raw_options |= htmlParserOption_HTML_PARSE_NOBLANKS;
        }

        // SAFETY: c_html 是有效的 CString，size 已验证
        // 调用 libxml2-sys 的安全封装函数
        let doc_ptr =
            unsafe { parse_html_memory(c_html.as_ptr(), size as i32, raw_options as i32) };

        if doc_ptr.is_null() {
            return Err(Error::ParseFailed { detail: None });
        }

        Ok(Self {
            doc_ptr,
            _marker: PhantomData,
        })
    }

    /// 从 XML 字符串解析文档
    ///
    /// # Arguments
    ///
    /// * `xml` - XML 字符串
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let xml = r#"<?xml version="1.0"?><root><item>data</item></root>"#;
    /// let doc = Document::parse_xml(xml).unwrap();
    /// ```
    pub fn parse_xml(xml: &str) -> Result<Self> {
        Self::parse_xml_with_options(xml, XmlParseOptions::default())
    }

    /// 使用自定义选项解析 XML 文档
    ///
    /// # Arguments
    ///
    /// * `xml` - XML 字符串
    /// * `options` - XML 解析选项
    pub fn parse_xml_with_options(xml: &str, options: XmlParseOptions) -> Result<Self> {
        // 检查输入大小
        let size = xml.len();
        if size > MAX_INPUT_SIZE {
            return Err(Error::InputTooLarge {
                size,
                max: MAX_INPUT_SIZE,
            });
        }

        let c_xml = CString::new(xml).map_err(|_| Error::NullByte)?;

        // 构建 XML 解析选项
        let mut raw_options: xmlParserOption = 0;
        if options.no_blanks {
            raw_options |= xmlParserOption_XML_PARSE_NOBLANKS;
        }
        if options.no_dtd {
            raw_options |= xmlParserOption_XML_PARSE_DTDLOAD;
        }

        // SAFETY: c_xml 是有效的 CString，size 已验证
        // 调用 libxml2-sys 的安全封装函数
        let doc_ptr = unsafe { parse_xml_memory(c_xml.as_ptr(), size as i32, raw_options as i32) };

        if doc_ptr.is_null() {
            return Err(Error::ParseFailed { detail: None });
        }

        Ok(Self {
            doc_ptr,
            _marker: PhantomData,
        })
    }

    /// 执行 XPath 查询并返回结果
    ///
    /// XPath 表达式可以返回多种类型：
    /// - 节点集合：`//div`
    /// - 布尔值：`count(//div) > 5`
    /// - 数字：`count(//div)`
    /// - 字符串：`string(//div/@class)`
    ///
    /// # Arguments
    ///
    /// * `xpath` - XPath 表达式
    ///
    /// # Returns
    ///
    /// 返回 XPath 求值结果，生命周期绑定到当前文档
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidXPath`] - XPath 表达式无效
    /// - [`Error::NullByte`] - XPath 包含空字节
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::{Document, XPathResult};
    ///
    /// let doc = Document::parse("<div><p>A</p><p>B</p></div>").unwrap();
    ///
    /// // 节点集合
    /// let result = doc.evaluate("//p").unwrap();
    /// if let XPathResult::NodeSet(nodes) = result {
    ///     println!("Found {} nodes", nodes.len());
    /// }
    ///
    /// // 数字
    /// let result = doc.evaluate("count(//p)").unwrap();
    /// if let XPathResult::Number(n) = result {
    ///     println!("Count: {}", n); // 2.0
    /// }
    ///
    /// // 布尔值
    /// let result = doc.evaluate("count(//p) > 1").unwrap();
    /// if let XPathResult::Boolean(b) = result {
    ///     println!("Has more than 1 p: {}", b); // true
    /// }
    ///
    /// // 字符串
    /// let result = doc.evaluate("string(//p)").unwrap();
    /// if let XPathResult::String(s) = result {
    ///     println!("First p text: {}", s); // "A"
    /// }
    /// ```
    pub fn evaluate<'a>(&'a self, xpath: &str) -> Result<XPathResult<'a>> {
        // evaluate_xpath 内部处理了 unsafe 操作
        evaluate_xpath(self.doc_ptr, xpath)
    }

    /// 执行 XPath 查询并返回匹配的节点（便捷方法）
    ///
    /// 这是 `evaluate()` 的便捷包装，只返回节点集合。
    /// 如果 XPath 返回非节点类型，返回空向量。
    ///
    /// # Arguments
    ///
    /// * `xpath` - XPath 表达式
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let doc = Document::parse("<div><p>A</p><p>B</p></div>").unwrap();
    /// let nodes = doc.select("//p").unwrap();
    /// println!("Found {} nodes", nodes.len());
    /// ```
    pub fn select<'a>(&'a self, xpath: &str) -> Result<Vec<SelectedNode<'a>>> {
        match self.evaluate(xpath)? {
            XPathResult::NodeSet(nodes) => Ok(nodes),
            _ => Ok(Vec::new()),
        }
    }

    /// 执行 XPath 查询并返回所有匹配节点的文本内容
    ///
    /// # Arguments
    ///
    /// * `xpath` - XPath 表达式
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let doc = Document::parse("<ul><li>Apple</li><li>Banana</li></ul>").unwrap();
    /// let texts = doc.extract_texts("//li").unwrap();
    /// assert_eq!(texts, vec!["Apple", "Banana"]);
    /// ```
    pub fn extract_texts(&self, xpath: &str) -> Result<Vec<String>> {
        let nodes = self.select(xpath)?;
        Ok(nodes.iter().map(|n| n.text()).collect())
    }

    /// 执行 XPath 查询并返回数字结果
    ///
    /// 适用于 `count()`, `sum()`, `number()` 等 XPath 函数
    ///
    /// # Arguments
    ///
    /// * `xpath` - XPath 表达式
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let doc = Document::parse("<div><p>A</p><p>B</p></div>").unwrap();
    /// let count = doc.extract_number("count(//p)").unwrap();
    /// assert_eq!(count, 2.0);
    /// ```
    pub fn extract_number(&self, xpath: &str) -> Result<f64> {
        match self.evaluate(xpath)? {
            XPathResult::Number(n) => Ok(n),
            XPathResult::Boolean(b) => Ok(if b { 1.0 } else { 0.0 }),
            XPathResult::String(s) => Ok(s.parse().unwrap_or(0.0)),
            _ => Err(Error::invalid_xpath(xpath)),
        }
    }

    /// 执行 XPath 查询并返回布尔结果
    ///
    /// 适用于比较表达式或 `boolean()` 函数
    ///
    /// # Arguments
    ///
    /// * `xpath` - XPath 表达式
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let doc = Document::parse("<div><p>A</p><p>B</p></div>").unwrap();
    /// let has_multiple = doc.extract_boolean("count(//p) > 1").unwrap();
    /// assert!(has_multiple);
    /// ```
    pub fn extract_boolean(&self, xpath: &str) -> Result<bool> {
        match self.evaluate(xpath)? {
            XPathResult::Boolean(b) => Ok(b),
            XPathResult::Number(n) => Ok(n != 0.0),
            XPathResult::NodeSet(nodes) => Ok(!nodes.is_empty()),
            XPathResult::String(s) => Ok(!s.is_empty()),
            _ => Err(Error::invalid_xpath(xpath)),
        }
    }

    /// 执行 XPath 查询并返回字符串结果
    ///
    /// 适用于 `string()`, `concat()`, `substring()` 等 XPath 函数
    ///
    /// # Arguments
    ///
    /// * `xpath` - XPath 表达式
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let doc = Document::parse("<div class=\"container\">Hello</div>").unwrap();
    /// let class = doc.extract_string("string(//div/@class)").unwrap();
    /// assert_eq!(class, "container");
    /// ```
    pub fn extract_string(&self, xpath: &str) -> Result<String> {
        match self.evaluate(xpath)? {
            XPathResult::String(s) => Ok(s),
            XPathResult::Number(n) => Ok(n.to_string()),
            XPathResult::Boolean(b) => Ok(b.to_string()),
            XPathResult::NodeSet(nodes) => {
                if nodes.is_empty() {
                    Ok(String::new())
                } else {
                    Ok(nodes[0].text())
                }
            }
            _ => Err(Error::invalid_xpath(xpath)),
        }
    }

    /// 获取文档根节点
    ///
    /// # Example
    ///
    /// ```
    /// use rlibxml::Document;
    ///
    /// let doc = Document::parse("<html><body>Test</body></html>").unwrap();
    /// let root = doc.root().unwrap();
    /// assert_eq!(root.tag_name(), "html");
    /// ```
    pub fn root(&self) -> Option<SelectedNode<'_>> {
        // SAFETY: doc_ptr 在 Document 存活期间始终有效
        // 使用 libxml2-sys 提供的安全封装函数
        let root = unsafe { doc_get_root_element(self.doc_ptr) };
        if root.is_null() {
            None
        } else {
            // SAFETY: root 是有效的节点指针，生命周期绑定到 self
            Some(unsafe { SelectedNode::from_raw(root) })
        }
    }

    /// 检查文档是否为空
    pub fn is_empty(&self) -> bool {
        self.root().is_none()
    }

    /// 获取原始文档指针（用于高级用途）
    ///
    /// # Safety
    ///
    /// 使用此指针时必须确保文档仍然存活。
    /// 不要通过此指针释放文档。
    #[inline]
    pub unsafe fn as_ptr(&self) -> xmlDocPtr {
        self.doc_ptr
    }
}

impl Drop for Document {
    fn drop(&mut self) {
        // SAFETY: doc_ptr 在 drop 时仍然有效
        // 使用 libxml2-sys 提供的安全封装函数
        unsafe {
            doc_free(self.doc_ptr);
            self.doc_ptr = ptr::null_mut();
        }
    }
}

impl std::fmt::Debug for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let root_info = self
            .root()
            .map(|r| r.tag_name())
            .unwrap_or_else(|| "(empty)".to_string());
        write!(f, "Document(root: {})", root_info)
    }
}
