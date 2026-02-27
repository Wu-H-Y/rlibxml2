//! xml-scraper - 安全的 HTML/XML 解析与 XPath 查询库
//!
//! 这个库提供了对 libxml2 的安全 Rust 封装，专门针对 Web 爬虫场景优化。
//!
//! # 特性
//!
//! - 零外部依赖：无需系统安装 libxml2
//! - 移动端友好：通过精简编译配置，避免交叉编译问题
//! - 内存安全：通过 Drop trait 自动管理 DOM 树内存
//! - 容错解析：专为处理真实世界的脏 HTML 设计
//!
//! # 示例
//!
//! ```rust,no_run
//! use xml_scraper::Document;
//!
//! let html = r#"
//!     <html>
//!         <body>
//!             <ul>
//!                 <li class="item">Apple</li>
//!                 <li class="item">Banana</li>
//!             </ul>
//!         </body>
//!     </html>
//! "#;
//!
//! let doc = Document::parse(html)?;
//! let items = doc.select("//li[@class='item']")?;
//!
//! for item in items {
//!     println!("Item: {}", item.text());
//! }
//! # Ok::<(), xml_scraper::Error>(())
//! ```

use libxml2_sys::*;
use std::ffi::{CStr, CString};
use std::ptr;

pub use error::Error;

mod error;

/// XPath 查询结果
///
/// XPath 表达式可以返回四种基本类型之一。
///
/// # 示例
///
/// ```rust,no_run
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
pub enum XPathResult {
    /// 节点集合
    NodeSet(Vec<SelectedNode>),
    /// 布尔值
    Boolean(bool),
    /// 数字（浮点数）
    Number(f64),
    /// 字符串
    String(String),
    /// 空结果或未知类型
    Empty,
}

/// 解析后的 XML/HTML 文档
///
/// 这是一个 RAII 类型，当它被 drop 时会自动释放整个 DOM 树。
///
/// # 示例
///
/// ```rust,no_run
/// use xml_scraper::Document;
///
/// // 解析 HTML（默认容错模式）
/// let doc = Document::parse("<div>Hello</div>")?;
///
/// // 解析 XML（严格模式）
/// let doc = Document::parse_xml("<root><item>data</item></root>")?;
///
/// // 使用自定义选项解析 HTML
/// use xml_scraper::ParseOptions;
/// let html = "<div>Hello</div>";
/// let doc = Document::parse_html_with_options(html, ParseOptions::default())?;
/// # Ok::<(), xml_scraper::Error>(())
/// ```
pub struct Document {
    doc_ptr: xmlDocPtr,
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
    /// - [`Error::ParseFailed`] - 解析失败
    pub fn parse(html: &str) -> Result<Self, Error> {
        Self::parse_html_with_options(html, ParseOptions::default())
    }

    /// 从 HTML 字符串解析文档（显式方法）
    ///
    /// 等价于 [`Document::parse`]，用于代码可读性。
    pub fn parse_html(html: &str) -> Result<Self, Error> {
        Self::parse(html)
    }

    /// 使用自定义选项解析 HTML 文档
    ///
    /// # Arguments
    ///
    /// * `html` - HTML 字符串
    /// * `options` - 解析选项
    pub fn parse_html_with_options(html: &str, options: ParseOptions) -> Result<Self, Error> {
        let c_html = CString::new(html).map_err(|_| Error::NullByte)?;

        unsafe {
            let mut raw_options: i32 = 0;

            if options.recover {
                raw_options |= htmlParserOption_HTML_PARSE_RECOVER as i32;
            }
            if options.no_error {
                raw_options |= htmlParserOption_HTML_PARSE_NOERROR as i32;
            }
            if options.no_warning {
                raw_options |= htmlParserOption_HTML_PARSE_NOWARNING as i32;
            }
            if options.no_blanks {
                raw_options |= htmlParserOption_HTML_PARSE_NOBLANKS as i32;
            }

            let doc_ptr = htmlReadMemory(
                c_html.as_ptr(),
                html.len() as i32,
                ptr::null(),
                b"UTF-8\0".as_ptr() as *const i8,
                raw_options,
            );

            if doc_ptr.is_null() {
                return Err(Error::ParseFailed);
            }

            Ok(Self { doc_ptr })
        }
    }

    /// 从 XML 字符串解析文档
    ///
    /// # Arguments
    ///
    /// * `xml` - XML 字符串
    pub fn parse_xml(xml: &str) -> Result<Self, Error> {
        let c_xml = CString::new(xml).map_err(|_| Error::NullByte)?;

        unsafe {
            let doc_ptr = xmlReadMemory(
                c_xml.as_ptr(),
                xml.len() as i32,
                ptr::null(),
                ptr::null(),
                0,
            );

            if doc_ptr.is_null() {
                return Err(Error::ParseFailed);
            }

            Ok(Self { doc_ptr })
        }
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
    /// 返回 XPath 求值结果
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidXPath`] - XPath 表达式无效
    /// - [`Error::NullByte`] - XPath 包含空字节
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use xml_scraper::{Document, XPathResult};
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
    pub fn evaluate(&self, xpath: &str) -> Result<XPathResult, Error> {
        let c_xpath = CString::new(xpath).map_err(|_| Error::NullByte)?;

        unsafe {
            let ctx = xmlXPathNewContext(self.doc_ptr);
            if ctx.is_null() {
                return Err(Error::XPathContextFailed);
            }

            let xpath_obj = xmlXPathEvalExpression(c_xpath.as_ptr() as *const xmlChar, ctx);

            if xpath_obj.is_null() {
                xmlXPathFreeContext(ctx);
                return Err(Error::InvalidXPath);
            }

            let result_type = (*xpath_obj).type_;
            let result = match result_type {
                // XPATH_NODESET = 1
                1 => {
                    let mut nodes = Vec::new();
                    let nodeset = (*xpath_obj).nodesetval;
                    if !nodeset.is_null() && (*nodeset).nodeNr > 0 {
                        let node_count = (*nodeset).nodeNr as isize;
                        let node_tab = (*nodeset).nodeTab;

                        for i in 0..node_count {
                            let node = *node_tab.offset(i);
                            if !node.is_null() {
                                nodes.push(SelectedNode {
                                    node_ptr: node,
                                    doc_ptr: self.doc_ptr,
                                });
                            }
                        }
                    }
                    XPathResult::NodeSet(nodes)
                }
                // XPATH_BOOLEAN = 2
                2 => XPathResult::Boolean((*xpath_obj).boolval != 0),
                // XPATH_NUMBER = 3
                3 => XPathResult::Number((*xpath_obj).floatval),
                // XPATH_STRING = 4
                4 => {
                    let stringval = (*xpath_obj).stringval;
                    if stringval.is_null() {
                        XPathResult::String(String::new())
                    } else {
                        let c_str = CStr::from_ptr(stringval as *const i8);
                        XPathResult::String(c_str.to_string_lossy().into_owned())
                    }
                }
                // 其他类型
                _ => XPathResult::Empty,
            };

            xmlXPathFreeObject(xpath_obj);
            xmlXPathFreeContext(ctx);

            Ok(result)
        }
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
    /// ```rust,no_run
    /// use xml_scraper::Document;
    ///
    /// let doc = Document::parse("<div><p>A</p><p>B</p></div>").unwrap();
    /// let nodes = doc.select("//p").unwrap();
    /// println!("Found {} nodes", nodes.len());
    /// ```
    pub fn select(&self, xpath: &str) -> Result<Vec<SelectedNode>, Error> {
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
    pub fn extract_texts(&self, xpath: &str) -> Result<Vec<String>, Error> {
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
    /// ```rust,no_run
    /// use xml_scraper::Document;
    ///
    /// let doc = Document::parse("<div><p>A</p><p>B</p></div>").unwrap();
    /// let count = doc.extract_number("count(//p)").unwrap();
    /// assert_eq!(count, 2.0);
    /// ```
    pub fn extract_number(&self, xpath: &str) -> Result<f64, Error> {
        match self.evaluate(xpath)? {
            XPathResult::Number(n) => Ok(n),
            XPathResult::Boolean(b) => Ok(if b { 1.0 } else { 0.0 }),
            XPathResult::String(s) => Ok(s.parse().unwrap_or(0.0)),
            _ => Err(Error::InvalidXPath),
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
    /// ```rust,no_run
    /// use xml_scraper::Document;
    ///
    /// let doc = Document::parse("<div><p>A</p><p>B</p></div>").unwrap();
    /// let has_multiple = doc.extract_boolean("count(//p) > 1").unwrap();
    /// assert!(has_multiple);
    /// ```
    pub fn extract_boolean(&self, xpath: &str) -> Result<bool, Error> {
        match self.evaluate(xpath)? {
            XPathResult::Boolean(b) => Ok(b),
            XPathResult::Number(n) => Ok(n != 0.0),
            XPathResult::NodeSet(nodes) => Ok(!nodes.is_empty()),
            XPathResult::String(s) => Ok(!s.is_empty()),
            _ => Err(Error::InvalidXPath),
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
    /// ```rust,no_run
    /// use xml_scraper::Document;
    ///
    /// let doc = Document::parse("<div class=\"container\">Hello</div>").unwrap();
    /// let class = doc.extract_string("string(//div/@class)").unwrap();
    /// assert_eq!(class, "container");
    /// ```
    pub fn extract_string(&self, xpath: &str) -> Result<String, Error> {
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
            _ => Err(Error::InvalidXPath),
        }
    }

    /// 获取文档根节点
    pub fn root(&self) -> Option<SelectedNode> {
        unsafe {
            let root = xmlDocGetRootElement(self.doc_ptr);
            if root.is_null() {
                None
            } else {
                Some(SelectedNode {
                    node_ptr: root,
                    doc_ptr: self.doc_ptr,
                })
            }
        }
    }

    /// 获取原始文档指针（用于高级用途）
    ///
    /// # Safety
    ///
    /// 使用此指针时必须确保文档仍然存活
    pub unsafe fn as_ptr(&self) -> xmlDocPtr {
        self.doc_ptr
    }
}

impl Drop for Document {
    fn drop(&mut self) {
        unsafe {
            if !self.doc_ptr.is_null() {
                xmlFreeDoc(self.doc_ptr);
            }
        }
    }
}

/// 查询选中的节点
///
/// 表示 XPath 查询结果中的一个节点
#[derive(Debug, Clone)]
pub struct SelectedNode {
    node_ptr: xmlNodePtr,
    #[allow(dead_code)]
    doc_ptr: xmlDocPtr,
}

impl SelectedNode {
    /// 获取节点的文本内容
    ///
    /// 对于文本节点，返回文本内容；对于元素节点，返回所有子文本的连接
    pub fn text(&self) -> String {
        unsafe {
            let text_ptr = xmlNodeGetContent(self.node_ptr);
            if text_ptr.is_null() {
                return String::new();
            }

            let c_str = CStr::from_ptr(text_ptr as *const i8);
            let result = c_str.to_string_lossy().into_owned();
            free_xml_char(text_ptr as *mut xmlChar);
            result
        }
    }

    /// 获取节点的标签名
    pub fn tag_name(&self) -> String {
        unsafe {
            if (*self.node_ptr).name.is_null() {
                return String::new();
            }
            let c_str = CStr::from_ptr((*self.node_ptr).name as *const i8);
            c_str.to_string_lossy().into_owned()
        }
    }

    /// 获取节点的 XPath 路径
    pub fn path(&self) -> String {
        unsafe {
            let path_ptr = xmlGetNodePath(self.node_ptr);
            if path_ptr.is_null() {
                return String::new();
            }
            let c_str = CStr::from_ptr(path_ptr as *const i8);
            let result = c_str.to_string_lossy().into_owned();
            free_xml_char(path_ptr as *mut xmlChar);
            result
        }
    }

    /// 获取节点的类型
    pub fn node_type(&self) -> NodeType {
        unsafe {
            match (*self.node_ptr).type_ {
                1 => NodeType::Element,
                2 => NodeType::Attribute,
                3 => NodeType::Text,
                4 => NodeType::CDataSection,
                5 => NodeType::EntityReference,
                6 => NodeType::Entity,
                7 => NodeType::ProcessingInstruction,
                8 => NodeType::Comment,
                9 => NodeType::Document,
                10 => NodeType::DocumentType,
                11 => NodeType::DocumentFragment,
                12 => NodeType::Notation,
                _ => NodeType::Unknown,
            }
        }
    }

    /// 获取原始节点指针
    pub unsafe fn as_ptr(&self) -> xmlNodePtr {
        self.node_ptr
    }
}

/// 节点类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Element,
    Attribute,
    Text,
    CDataSection,
    EntityReference,
    Entity,
    ProcessingInstruction,
    Comment,
    Document,
    DocumentType,
    DocumentFragment,
    Notation,
    Unknown,
}

/// HTML 解析选项
#[derive(Debug, Clone, Copy)]
pub struct ParseOptions {
    /// 启用容错模式，尝试解析破损的 HTML
    pub recover: bool,
    /// 抑制错误输出
    pub no_error: bool,
    /// 抑制警告输出
    pub no_warning: bool,
    /// 移除空白节点
    pub no_blanks: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            recover: true,
            no_error: true,
            no_warning: true,
            no_blanks: false,
        }
    }
}

/// 初始化 libxml2 解析器（通常不需要）
///
/// # 何时需要调用
///
/// - **单线程应用**：不需要（libxml2 会自动惰性初始化）
/// - **多线程应用**：建议在启动时调用一次，确保线程安全
///
/// # 何时不需要调用
///
/// - 大多数情况下不需要
/// - 每个文档的内存由 `Document` 的 `Drop` 自动管理
/// - 不需要每次解析前调用
///
/// # Example
/// ```rust,no_run
/// use xml_scraper::{Document, init};
///
/// fn main() {
///     init(); // 多线程环境建议调用，单线程可省略
///
///     let doc = Document::parse("<div>Hello</div>").unwrap();
///     // doc 自动释放
/// }
/// ```
pub fn init() {
    libxml2_sys::init_parser();
}

/// 清理 libxml2 解析器（几乎从不需要）
///
/// # 重要
///
/// - **几乎从不需要调用此函数**
/// - 进程退出时操作系统会自动回收所有内存
/// - 这是全局清理，调用后不能再解析任何文档
///
/// # 唯一需要的场景
///
/// 如果你的程序需要在运行时动态卸载 libxml2 相关的动态库（极少见），
/// 才需要调用此函数。
pub fn cleanup() {
    libxml2_sys::cleanup_parser();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_parse() {
        let html = r#"
            <html>
                <body>
                    <ul>
                        <li class="item">Apple</li>
                        <li class="item">Banana</li>
                        <li class="item">Cherry</li>
                    </ul>
                </body>
            </html>
        "#;

        let doc = Document::parse(html).unwrap();
        let items = doc.extract_texts("//li[@class='item']").unwrap();

        assert_eq!(items, vec!["Apple", "Banana", "Cherry"]);
    }

    #[test]
    fn test_broken_html() {
        // 测试容错解析
        let broken_html = r#"
            <div>
                <p>Unclosed paragraph
                <p>Another one
                <ul>
                    <li>Item 1
                    <li>Item 2
                </ul>
            </div>
        "#;

        let doc = Document::parse(broken_html).unwrap();
        let items = doc.extract_texts("//li").unwrap();

        // HTML 解析会保留源码中的空白，使用 trim() 来比较
        let trimmed: Vec<String> = items.iter().map(|s| s.trim().to_string()).collect();
        assert_eq!(trimmed, vec!["Item 1", "Item 2"]);
    }

    #[test]
    fn test_node_path() {
        let html = r#"<div><p><span>Hello</span></p></div>"#;
        let doc = Document::parse(html).unwrap();

        let nodes = doc.select("//span").unwrap();
        assert_eq!(nodes.len(), 1);
        assert!(nodes[0].path().contains("span"));
    }

    #[test]
    fn test_root_element() {
        let html = r#"<html><body>Test</body></html>"#;
        let doc = Document::parse(html).unwrap();

        let root = doc.root().unwrap();
        assert_eq!(root.tag_name(), "html");
    }

    #[test]
    fn test_node_type() {
        let html = r#"<p>Text<!-- comment --></p>"#;
        let doc = Document::parse(html).unwrap();

        let elements = doc.select("//p").unwrap();
        assert_eq!(elements[0].node_type(), NodeType::Element);

        let text_nodes = doc.select("//p/text()").unwrap();
        assert!(!text_nodes.is_empty());
        for node in text_nodes {
            assert_eq!(node.node_type(), NodeType::Text);
        }
    }

    #[test]
    fn test_parse_xml() {
        let xml = r#"<?xml version="1.0"?><root><item>data</item></root>"#;
        let doc = Document::parse_xml(xml).unwrap();

        let items = doc.extract_texts("//item").unwrap();
        assert_eq!(items, vec!["data"]);
    }
}
