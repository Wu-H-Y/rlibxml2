//! rlibxml - 安全的 HTML/XML 解析与 XPath 查询库
//!
//! 这个库提供了对 libxml2 的安全 Rust 封装，专门针对 Web 爬虫场景优化。
//!
//! # 特性
//!
//! - **零外部依赖**：无需系统安装 libxml2，源码编译静态链接
//! - **移动端友好**：通过精简编译配置，避免交叉编译问题
//! - **内存安全**：通过生命周期绑定，确保节点不会超出文档生命周期
//! - **容错解析**：专为处理真实世界的脏 HTML 设计
//! - **完整功能**：属性访问、节点遍历、XPath 查询
//!
//! # 快速开始
//!
//! ```rust
//! use rlibxml::Document;
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
//! # Ok::<(), rlibxml::Error>(())
//! ```
//!
//! # 生命周期安全
//!
//! 节点引用（`SelectedNode`）的生命周期绑定到所属的 `Document`，
//! Rust 编译器会在编译时阻止悬垂引用：
//!
//! ```compile_fail
//! use rlibxml::Document;
//!
//! let node = {
//!     let doc = Document::parse("<div>test</div>").unwrap();
//!     let node = doc.select("//div").unwrap()[0].clone();
//!     node
//!     // doc 在此处被 drop
//! };
//! node.text();  // 编译错误：`doc` 的生命周期不够长
//! ```
//!
//! # API 概览
//!
//! ## 文档解析
//!
//! - [`Document::parse`] - 解析 HTML（容错模式）
//! - [`Document::parse_xml`] - 解析 XML（严格模式）
//! - [`Document::parse_html_with_options`] - 使用自定义选项解析
//!
//! ## XPath 查询
//!
//! - [`Document::select`] - 查询节点
//! - [`Document::evaluate`] - 查询并返回任意类型结果
//! - [`Document::extract_texts`] - 提取所有匹配节点的文本
//! - [`Document::extract_number`] - 提取数字结果
//! - [`Document::extract_boolean`] - 提取布尔结果
//!
//! ## 节点操作
//!
//! - [`SelectedNode::text`] - 获取文本内容
//! - [`SelectedNode::tag_name`] - 获取标签名
//! - [`SelectedNode::attr`] - 获取属性值
//! - [`SelectedNode::attrs`] - 获取所有属性
//! - [`SelectedNode::children`] - 获取子节点
//! - [`SelectedNode::parent`] - 获取父节点
//! - [`SelectedNode::select`] - 在节点上下文中查询

mod document;
mod error;
mod node;
mod node_type;
mod options;
mod xpath;

// 重导出公共 API
pub use document::Document;
pub use error::{Error, Result};
pub use node::SelectedNode;
pub use node_type::NodeType;
pub use options::{ParseOptions, XmlParseOptions};
pub use xpath::XPathResult;

// ========================================
// 全局函数
// ========================================

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
///
/// ```rust
/// use rlibxml::{Document, init};
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
