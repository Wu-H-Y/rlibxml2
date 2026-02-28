//! 边界情况测试

use xml_scraper::{Document, NodeType, ParseOptions, XPathResult};

/// 测试最小有效 HTML
#[test]
fn test_minimal_html() {
    let cases = vec![
        ("<p>", "single unclosed tag"),
        ("<p/>", "self-closing tag"),
        ("text", "plain text"),
        ("<>", "empty tag"),
        ("< >", "whitespace tag"),
    ];

    for (html, desc) in cases {
        let result = Document::parse(html);
        assert!(result.is_ok(), "Failed for case: {}", desc);
    }
}

/// 测试极端属性情况
#[test]
fn test_extreme_attributes() {
    // 大量属性
    let mut html = String::from("<div ");
    for i in 0..100 {
        html.push_str(&format!("attr{}=\"value{}\" ", i, i));
    }
    html.push_str(">content</div>");

    let doc = Document::parse(&html).unwrap();
    let div = &doc.select("//div").unwrap()[0];

    let attrs = div.attrs();
    assert_eq!(attrs.len(), 100);

    // 特殊属性名
    let html = r#"<div data-test="1" data_test="2" data:test="3" data-test-abc="4">x</div>"#;
    let doc = Document::parse(html).unwrap();
    let div = &doc.select("//div").unwrap()[0];
    assert!(div.has_attr("data-test"));
}

/// 测试特殊字符处理
#[test]
fn test_special_characters() {
    let html = r#"<div>
        <p id="quotes">Text with "double" and 'single' quotes</p>
        <p id="backslash">Path: C:\Users\test</p>
        <p id="newlines">Line1
Line2
Line3</p>
        <p id="tabs">Col1	Col2	Col3</p>
        <p id="mixed">Mix of <>&"' chars</p>
    </div>"#;

    let doc = Document::parse(html).unwrap();

    let quotes = doc.extract_string("//p[@id='quotes']").unwrap();
    assert!(quotes.contains("double"));

    let backslash = doc.extract_string("//p[@id='backslash']").unwrap();
    assert!(backslash.contains("\\"));

    let newlines = doc.extract_string("//p[@id='newlines']").unwrap();
    assert!(newlines.contains('\n'));
}

/// 测试 Unicode 边界情况
#[test]
fn test_unicode_edge_cases() {
    let html = r#"<div>
        <p id="emoji">👨‍👩‍👧‍👦 Family emoji</p>
        <p id="rtl">مرحبا بالعالم</p>
        <p id="mixed-rtl">Hello مرحبا World</p>
        <p id="zero-width">a\u{200B}b\u{200B}c</p>
        <p id="combining">é ñ ü</p>
    </div>"#;

    let doc = Document::parse(html).unwrap();

    let emoji = doc.extract_string("//p[@id='emoji']").unwrap();
    assert!(emoji.contains("👨‍👩‍👧‍👦"));

    let rtl = doc.extract_string("//p[@id='rtl']").unwrap();
    assert!(!rtl.is_empty());
}

/// 测试 XPath 边界情况
#[test]
fn test_xpath_edge_cases() {
    let doc = Document::parse("<div><p>A</p><p>B</p></div>").unwrap();

    // 空结果
    let result = doc.select("//nonexistent").unwrap();
    assert!(result.is_empty());

    // 结果类型转换
    let result = doc.evaluate("count(//p)").unwrap();
    assert!(result.as_boolean()); // 2.0 != 0
    assert_eq!(result.as_string(), "2");

    let result = doc.evaluate("1 = 1").unwrap();
    assert_eq!(result.as_number(), 1.0);

    // 空节点集合
    let result = doc.evaluate("//nonexistent").unwrap();
    if let XPathResult::NodeSet(nodes) = result {
        assert!(nodes.is_empty());
        assert_eq!(nodes.len(), 0);
    }
}

/// 测试节点遍历边界
#[test]
fn test_traversal_boundaries() {
    let html = "<div><p>A</p></div>";
    let doc = Document::parse(html).unwrap();

    // 无兄弟节点时的 next/prev
    let p = &doc.select("//p").unwrap()[0];
    assert!(p.next_sibling().is_none() || p.next_sibling().unwrap().node_type() == NodeType::Text);
    assert!(p.prev_sibling().is_none() || p.prev_sibling().unwrap().node_type() == NodeType::Text);

    // 根节点存在
    let root = doc.root();
    assert!(root.is_some());
}

/// 测试文档边界状态
#[test]
fn test_document_boundary_states() {
    // 仅空白
    let _doc = Document::parse("   ").unwrap();
    // 可能是空文档或有文本节点

    // 仅注释
    let _doc = Document::parse("<!-- comment -->").unwrap();

    // 仅 DOCTYPE
    let _doc = Document::parse("<!DOCTYPE html>").unwrap();

    // 混合无效标签
    let doc = Document::parse("</div><div>test</div>").unwrap();
    let result = doc.select("//div").unwrap();
    assert!(!result.is_empty());
}

/// 测试解析选项边界
#[test]
fn test_parse_options_boundaries() {
    let html = "<div>test</div>";

    // 所有选项关闭
    let options = ParseOptions {
        recover: false,
        no_error: false,
        no_warning: false,
        no_blanks: false,
    };
    let doc = Document::parse_html_with_options(html, options).unwrap();
    assert!(!doc.is_empty());

    // 所有选项开启
    let options = ParseOptions {
        recover: true,
        no_error: true,
        no_warning: true,
        no_blanks: true,
    };
    let doc = Document::parse_html_with_options(html, options).unwrap();
    assert!(!doc.is_empty());
}

/// 测试连续操作稳定性
#[test]
fn test_continuous_operations() {
    let doc = Document::parse("<div><p id='test'>content</p></div>").unwrap();

    // 连续 1000 次操作
    for i in 0..1000 {
        let xpath = if i % 2 == 0 { "//p" } else { "//div" };
        let result = doc.select(xpath).unwrap();
        assert!(!result.is_empty());

        let node = &result[0];
        let _ = node.text();
        let _ = node.tag_name();
        let _ = node.path();
        let _ = node.attrs();
    }
}

/// 测试内存压力
#[test]
fn test_memory_pressure() {
    // 创建并立即丢弃大量文档
    for _ in 0..100 {
        let large_html = format!("<div>{}</div>", "x".repeat(10000));
        let doc = Document::parse(&large_html).unwrap();
        let _ = doc.select("//div").unwrap();
        // doc dropped here
    }
}

/// 测试嵌套节点选择
#[test]
fn test_nested_selections() {
    let html = r#"<div>
        <section>
            <article>
                <p>Deep <span>content</span></p>
            </article>
        </section>
    </div>"#;

    let doc = Document::parse(html).unwrap();

    // 多层嵌套查询
    let sections = doc.select("//section").unwrap();
    for section in &sections {
        let articles = section.select(".//article").unwrap();
        for article in &articles {
            let paragraphs = article.select(".//p").unwrap();
            for p in &paragraphs {
                let spans = p.select(".//span").unwrap();
                assert_eq!(spans.len(), 1);
                assert_eq!(spans[0].text(), "content");
            }
        }
    }
}

/// 测试空属性值
#[test]
fn test_empty_attribute_values() {
    let html = r#"<div empty="" boolean no-value=>"content</div>"#;
    let doc = Document::parse(html).unwrap();

    let div = &doc.select("//div").unwrap()[0];

    // 空字符串属性
    if div.has_attr("empty") {
        let val = div.attr("empty").unwrap();
        assert_eq!(val, "");
    }

    // 布尔属性（无值）
    if div.has_attr("boolean") {
        let val = div.attr("boolean");
        // 布尔属性可能返回空字符串或属性名
        assert!(val.is_some());
    }
}

/// 测试错误信息质量
#[test]
fn test_error_message_quality() {
    // NullByte 错误
    let err = Document::parse("hello\0world").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("null byte") || msg.contains("NullByte"));

    // InvalidXPath 错误
    let doc = Document::parse("<div>test</div>").unwrap();
    let err = doc.select("//[invalid").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("XPath") || msg.contains("xpath"));
}

/// 测试混合内容节点
#[test]
fn test_mixed_content_nodes() {
    let html = r#"<p>This is <b>bold</b> and <i>italic</i> text.</p>"#;
    let doc = Document::parse(html).unwrap();

    let p = &doc.select("//p").unwrap()[0];

    // 获取完整文本
    let text = p.text();
    assert!(text.contains("bold"));
    assert!(text.contains("italic"));

    // 获取子元素
    let children = p.element_children();
    assert!(children.len() >= 2);
}

/// 测试自闭合标签
#[test]
fn test_self_closing_tags() {
    let html = r#"<div>
        <br/>
        <hr/>
        <img src="test.jpg"/>
        <input type="text"/>
    </div>"#;

    let doc = Document::parse(html).unwrap();

    let br = doc.select("//br").unwrap();
    assert_eq!(br.len(), 1);

    let img = &doc.select("//img").unwrap()[0];
    assert_eq!(img.attr("src"), Some("test.jpg".to_string()));
}
