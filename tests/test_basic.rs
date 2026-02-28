//! 基础解析测试

use rlibxml::Document;

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

    let trimmed: Vec<String> = items.iter().map(|s| s.trim().to_string()).collect();
    assert_eq!(trimmed, vec!["Item 1", "Item 2"]);
}

#[test]
fn test_parse_xml() {
    let xml = r#"<?xml version="1.0"?><root><item>data</item></root>"#;
    let doc = Document::parse_xml(xml).unwrap();

    let items = doc.extract_texts("//item").unwrap();
    assert_eq!(items, vec!["data"]);
}

#[test]
fn test_root_element() {
    let html = r#"<html><body>Test</body></html>"#;
    let doc = Document::parse(html).unwrap();

    let root = doc.root().unwrap();
    assert_eq!(root.tag_name(), "html");
}

#[test]
fn test_is_empty() {
    let doc = Document::parse("").unwrap();
    // 空输入应该产生空文档或带默认根节点的文档
    // 实际行为取决于 libxml2
    assert!(doc.is_empty() || doc.root().is_some());
}
