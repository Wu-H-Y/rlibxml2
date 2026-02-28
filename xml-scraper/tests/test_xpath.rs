//! XPath 查询测试

use xml_scraper::{Document, XPathResult};

#[test]
fn test_xpath_result_types() {
    let html = r#"<div><p>A</p><p>B</p></div>"#;
    let doc = Document::parse(html).unwrap();

    // 节点集合
    let result = doc.evaluate("//p").unwrap();
    assert!(result.is_nodeset());
    if let XPathResult::NodeSet(nodes) = result {
        assert_eq!(nodes.len(), 2);
    }

    // 数字
    let result = doc.evaluate("count(//p)").unwrap();
    assert!(result.is_number());
    assert_eq!(result.as_number(), 2.0);

    // 布尔值
    let result = doc.evaluate("count(//p) > 1").unwrap();
    assert!(result.is_boolean());
    assert!(result.as_boolean());

    // 字符串
    let result = doc.evaluate("string(//p)").unwrap();
    assert!(result.is_string());
    assert_eq!(result.as_string(), "A");
}

#[test]
fn test_node_select() {
    let html = r#"<div><p class="a">A</p><p class="b">B</p></div>"#;
    let doc = Document::parse(html).unwrap();

    let div = &doc.select("//div").unwrap()[0];
    let paragraphs = div.select(".//p").unwrap();
    assert_eq!(paragraphs.len(), 2);
}

#[test]
fn test_xpath_extract_convenience_methods() {
    let html = r#"<div><p>A</p><p>B</p></div>"#;
    let doc = Document::parse(html).unwrap();

    // extract_texts
    let texts = doc.extract_texts("//p").unwrap();
    assert_eq!(texts, vec!["A", "B"]);

    // extract_number
    let count = doc.extract_number("count(//p)").unwrap();
    assert_eq!(count, 2.0);

    // extract_boolean
    let has_multiple = doc.extract_boolean("count(//p) > 1").unwrap();
    assert!(has_multiple);

    // extract_string
    let first_text = doc.extract_string("string(//p)").unwrap();
    assert_eq!(first_text, "A");
}

#[test]
fn test_xpath_with_predicates() {
    let html = r#"
        <ul>
            <li class="item">First</li>
            <li class="item">Second</li>
            <li class="other">Third</li>
        </ul>
    "#;
    let doc = Document::parse(html).unwrap();

    let items = doc.select("//li[@class='item']").unwrap();
    assert_eq!(items.len(), 2);

    let first = doc.select("//li[1]").unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].text().trim(), "First");
}

#[test]
fn test_xpath_axis() {
    let html = r#"<div><p>A</p><span>B</span></div>"#;
    let doc = Document::parse(html).unwrap();

    // 子轴
    let children = doc.select("//div/child::*").unwrap();
    assert_eq!(children.len(), 2);

    // 后代轴
    let descendants = doc.select("//div/descendant::*").unwrap();
    assert!(descendants.len() >= 2);
}

#[test]
fn test_xpath_functions() {
    let html = r#"<div><p>Hello</p><p>World</p></div>"#;
    let doc = Document::parse(html).unwrap();

    // concat
    let result = doc
        .extract_string("concat(//p[1]/text(), ' ', //p[2]/text())")
        .unwrap();
    assert_eq!(result.trim(), "Hello World");

    // string-length
    let len = doc.extract_number("string-length(//p[1])").unwrap();
    assert_eq!(len, 5.0);

    // contains
    let has_hello = doc.extract_boolean("contains(//p[1], 'Hello')").unwrap();
    assert!(has_hello);
}
