//! HTML/XML 解析选项

/// HTML 解析选项
///
/// 控制解析器的行为，特别适合处理真实世界中的脏 HTML。
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
#[derive(Debug, Clone, Copy)]
pub struct ParseOptions {
    /// 启用容错模式，尝试解析破损的 HTML
    ///
    /// 对于 Web 爬虫场景，建议启用此选项。
    pub recover: bool,

    /// 抑制错误输出
    ///
    /// 启用后，解析器的错误消息不会输出到 stderr。
    pub no_error: bool,

    /// 抑制警告输出
    ///
    /// 启用后，解析器的警告消息不会输出到 stderr。
    pub no_warning: bool,

    /// 移除空白节点
    ///
    /// 启用后，仅包含空白的文本节点将被移除。
    /// 这可以简化 DOM 树，但可能影响 XPath 查询结果。
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

impl ParseOptions {
    /// 创建严格模式选项
    ///
    /// 不进行错误恢复，报告所有错误和警告。
    pub fn strict() -> Self {
        Self {
            recover: false,
            no_error: false,
            no_warning: false,
            no_blanks: false,
        }
    }

    /// 创建爬虫模式选项（默认）
    ///
    /// 最大容错，静默处理错误，适合处理真实世界的脏 HTML。
    pub fn scraper() -> Self {
        Self::default()
    }

    /// 创建紧凑模式选项
    ///
    /// 移除空白节点，生成更简洁的 DOM 树。
    pub fn compact() -> Self {
        Self {
            recover: true,
            no_error: true,
            no_warning: true,
            no_blanks: true,
        }
    }
}

/// XML 解析选项
#[derive(Debug, Clone, Copy)]
pub struct XmlParseOptions {
    /// 移除空白节点
    pub no_blanks: bool,

    /// 不加载外部 DTD
    pub no_dtd: bool,

    /// 不加载外部实体
    pub no_ent: bool,
}

impl Default for XmlParseOptions {
    fn default() -> Self {
        Self {
            no_blanks: false,
            no_dtd: true,
            no_ent: true,
        }
    }
}
