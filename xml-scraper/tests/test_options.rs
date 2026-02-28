//! 解析选项测试

use xml_scraper::{Document, ParseOptions, XmlParseOptions};

#[test]
fn test_parse_options() {
    let html = r#"<div>Hello</div>"#;

    // 测试默认选项
    let doc = Document::parse_html_with_options(html, ParseOptions::default()).unwrap();
    assert!(!doc.is_empty());

    // 测试严格选项
    let doc = Document::parse_html_with_options(html, ParseOptions::strict()).unwrap();
    assert!(!doc.is_empty());

    // 测试紧凑选项
    let doc = Document::parse_html_with_options(html, ParseOptions::compact()).unwrap();
    assert!(!doc.is_empty());
}

#[test]
fn test_custom_parse_options() {
    let html = r#"<div>Hello</div>"#;

    let options = ParseOptions {
        recover: true,
        no_error: true,
        no_warning: true,
        no_blanks: true,
    };

    let doc = Document::parse_html_with_options(html, options).unwrap();
    assert!(!doc.is_empty());
}

#[test]
fn test_xml_parse_options() {
    let xml = r#"<?xml version="1.0"?><root><item>data</item></root>"#;

    // 测试默认 XML 选项
    let doc = Document::parse_xml_with_options(xml, XmlParseOptions::default()).unwrap();
    assert!(!doc.is_empty());

    // 测试自定义 XML 选项
    let options = XmlParseOptions {
        no_blanks: true,
        no_dtd: false,
        no_ent: true,
    };

    let doc = Document::parse_xml_with_options(xml, options).unwrap();
    assert!(!doc.is_empty());
}

#[test]
fn test_broken_html_with_recover() {
    let broken_html = r#"<div><p>Unclosed<div>Nested</div>"#;

    // 使用恢复选项
    let options = ParseOptions {
        recover: true,
        no_error: true,
        no_warning: true,
        no_blanks: false,
    };

    let doc = Document::parse_html_with_options(broken_html, options).unwrap();
    // 应该成功解析并恢复
    assert!(!doc.is_empty());
}

#[test]
fn test_no_blanks_option() {
    let html = r#"<div>
        <p>Text</p>
    </div>"#;

    // 不去除空白节点
    let doc_with_blanks = Document::parse_html_with_options(
        html,
        ParseOptions {
            no_blanks: false,
            ..Default::default()
        },
    )
    .unwrap();

    // 去除空白节点
    let doc_no_blanks = Document::parse_html_with_options(
        html,
        ParseOptions {
            no_blanks: true,
            ..Default::default()
        },
    )
    .unwrap();

    // 两个文档都应该成功解析
    assert!(!doc_with_blanks.is_empty());
    assert!(!doc_no_blanks.is_empty());
}
