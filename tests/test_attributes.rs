//! 属性访问测试

use rlibxml::Document;

#[test]
fn test_attributes() {
    let html = r#"<div class="container" id="main" data-value="test">Hello</div>"#;
    let doc = Document::parse(html).unwrap();

    let node = &doc.select("//div").unwrap()[0];

    // 测试单个属性
    assert_eq!(node.attr("class"), Some("container".to_string()));
    assert_eq!(node.attr("id"), Some("main".to_string()));
    assert_eq!(node.attr("data-value"), Some("test".to_string()));
    assert_eq!(node.attr("nonexistent"), None);

    // 测试 has_attr
    assert!(node.has_attr("class"));
    assert!(!node.has_attr("style"));

    // 测试 attrs
    let attrs = node.attrs();
    assert_eq!(attrs.len(), 3);
    assert_eq!(attrs.get("class"), Some(&"container".to_string()));
}

#[test]
fn test_attr_empty_name() {
    let html = r#"<div class="test">Hello</div>"#;
    let doc = Document::parse(html).unwrap();

    let node = &doc.select("//div").unwrap()[0];

    // 空属性名应返回 None
    assert_eq!(node.attr(""), None);
    assert!(!node.has_attr(""));
}

#[test]
fn test_attrs_iteration() {
    let html = r#"<div a="1" b="2" c="3">Hello</div>"#;
    let doc = Document::parse(html).unwrap();

    let node = &doc.select("//div").unwrap()[0];
    let attrs = node.attrs();

    // 验证所有属性都被正确获取
    assert_eq!(attrs.get("a"), Some(&"1".to_string()));
    assert_eq!(attrs.get("b"), Some(&"2".to_string()));
    assert_eq!(attrs.get("c"), Some(&"3".to_string()));
}

#[test]
fn test_special_chars_in_attr() {
    let html = r#"<div data-json='{"key":"value"}'>Hello</div>"#;
    let doc = Document::parse(html).unwrap();

    let node = &doc.select("//div").unwrap()[0];
    let attr = node.attr("data-json");

    assert!(attr.is_some());
    assert!(attr.unwrap().contains("key"));
}
