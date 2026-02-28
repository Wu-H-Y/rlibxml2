//! 节点类型定义

/// 节点类型
///
/// 表示 XML/HTML DOM 树中节点的类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum NodeType {
    /// 元素节点 (1)
    Element = 1,
    /// 属性节点 (2)
    Attribute = 2,
    /// 文本节点 (3)
    Text = 3,
    /// CDATA 节点 (4)
    CDataSection = 4,
    /// 实体引用节点 (5)
    EntityReference = 5,
    /// 实体节点 (6)
    Entity = 6,
    /// 处理指令节点 (7)
    ProcessingInstruction = 7,
    /// 注释节点 (8)
    Comment = 8,
    /// 文档节点 (9)
    Document = 9,
    /// 文档类型节点 (10)
    DocumentType = 10,
    /// 文档片段节点 (11)
    DocumentFragment = 11,
    /// 符号节点 (12)
    Notation = 12,
    /// 未知类型
    Unknown = 0,
}

impl NodeType {
    /// 从 libxml2 的类型值转换
    #[inline]
    pub(crate) fn from_raw(value: i32) -> Self {
        match value {
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

    /// 转换为 libxml2 的类型值
    #[inline]
    pub fn to_raw(self) -> i32 {
        self as i32
    }

    /// 检查是否为元素节点
    #[inline]
    pub fn is_element(&self) -> bool {
        matches!(self, NodeType::Element)
    }

    /// 检查是否为文本节点（包括 CDATA）
    #[inline]
    pub fn is_text(&self) -> bool {
        matches!(self, NodeType::Text | NodeType::CDataSection)
    }

    /// 检查是否为属性节点
    #[inline]
    pub fn is_attribute(&self) -> bool {
        matches!(self, NodeType::Attribute)
    }

    /// 检查是否为注释节点
    #[inline]
    pub fn is_comment(&self) -> bool {
        matches!(self, NodeType::Comment)
    }

    /// 获取节点类型名称
    pub fn name(&self) -> &'static str {
        match self {
            NodeType::Element => "element",
            NodeType::Attribute => "attribute",
            NodeType::Text => "text",
            NodeType::CDataSection => "cdata",
            NodeType::EntityReference => "entity-reference",
            NodeType::Entity => "entity",
            NodeType::ProcessingInstruction => "processing-instruction",
            NodeType::Comment => "comment",
            NodeType::Document => "document",
            NodeType::DocumentType => "document-type",
            NodeType::DocumentFragment => "document-fragment",
            NodeType::Notation => "notation",
            NodeType::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
