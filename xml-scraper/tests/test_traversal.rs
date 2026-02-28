//! 节点遍历测试

use xml_scraper::Document;

#[test]
fn test_traversal() {
    let html = r#"<div><p id="a">A</p><p id="b">B</p><p id="c">C</p></div>"#;
    let doc = Document::parse(html).unwrap();

    let div = &doc.select("//div").unwrap()[0];

    // 测试子节点
    let children = div.element_children();
    assert_eq!(children.len(), 3);

    // 测试 first_child
    let first_p = div.select("./p[@id='a']").unwrap()[0].clone();

    // 测试 parent
    let parent = first_p.parent().unwrap();
    assert_eq!(parent.tag_name(), "div");

    // 测试 siblings
    let middle_p = &doc.select("//p[@id='b']").unwrap()[0];
    let siblings = middle_p.siblings();
    assert_eq!(siblings.len(), 2);
}

#[test]
fn test_next_prev_sibling() {
    let html = r#"<div><p id="a">A</p><p id="b">B</p><p id="c">C</p></div>"#;
    let doc = Document::parse(html).unwrap();

    let first = &doc.select("//p[@id='a']").unwrap()[0];
    let next = first.next_sibling().unwrap();
    assert_eq!(next.attr("id"), Some("b".to_string()));

    let last = &doc.select("//p[@id='c']").unwrap()[0];
    let prev = last.prev_sibling().unwrap();
    assert_eq!(prev.attr("id"), Some("b".to_string()));
}

#[test]
fn test_first_last_child() {
    let html = r#"<div><p id="a">A</p><p id="b">B</p><p id="c">C</p></div>"#;
    let doc = Document::parse(html).unwrap();

    let div = &doc.select("//div").unwrap()[0];

    // 找到第一个元素子节点
    let mut first_element = None;
    let mut child = div.first_child();
    while let Some(c) = child {
        if c.node_type().is_element() {
            first_element = Some(c);
            break;
        }
        child = c.next_sibling();
    }

    assert!(first_element.is_some());
    let first = first_element.unwrap();
    assert_eq!(first.attr("id"), Some("a".to_string()));
}

#[test]
fn test_element_children() {
    let html = r#"<div>text<p>A</p>more<p>B</p></div>"#;
    let doc = Document::parse(html).unwrap();

    let div = &doc.select("//div").unwrap()[0];
    let elements = div.element_children();
    assert_eq!(elements.len(), 2);
}

#[test]
fn test_text_children() {
    let html = r#"<div>Hello<p>A</p>World</div>"#;
    let doc = Document::parse(html).unwrap();

    let div = &doc.select("//div").unwrap()[0];
    let texts = div.text_children();
    // 文本子节点数量取决于解析器如何处理空白
    assert!(!texts.is_empty());
}
