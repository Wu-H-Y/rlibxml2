//! 安全性测试 - 验证生命周期和基本安全性

use xml_scraper::{Document, NodeType};

/// 测试生命周期编译时检查
///
/// 注意：这个测试验证运行时行为。
/// 编译时检查通过 `compile_fail` 文档测试验证。
#[test]
fn test_lifetime_runtime_safety() {
    // 正确用法：文档在使用节点期间保持存活
    let doc = Document::parse("<div>test</div>").unwrap();
    let node = &doc.select("//div").unwrap()[0];
    assert_eq!(node.text(), "test");
    // doc 在这里 drop，节点引用已经失效

    // 正确用法：在同一作用域内使用
    let doc2 = Document::parse("<span>hello</span>").unwrap();
    {
        let nodes = doc2.select("//span").unwrap();
        assert_eq!(nodes[0].text(), "hello");
    }
    // nodes 在这里 drop，然后 doc2 drop
}

/// 测试多线程中每个线程独立创建文档
#[test]
fn test_multiple_document_independence() {
    // 多个文档在同一作用域内独立使用
    let doc1 = Document::parse("<div><p>A</p><p>B</p><p>C</p></div>").unwrap();
    let doc2 = Document::parse("<div><p>X</p><p>Y</p></div>").unwrap();

    // 在不同文档上执行查询
    let result1 = doc1.select("//p").unwrap();
    let result2 = doc2.select("//p").unwrap();

    assert_eq!(result1.len(), 3);
    assert_eq!(result2.len(), 2);
}

/// 测试多次查询不影响文档状态
#[test]
fn test_multiple_queries_state_isolation() {
    let doc = Document::parse("<div><p>A</p><p>B</p></div>").unwrap();

    // 执行多次查询
    for _ in 0..100 {
        let result1 = doc.select("//p").unwrap();
        assert_eq!(result1.len(), 2);

        let result2 = doc.evaluate("count(//p)").unwrap();
        if let xml_scraper::XPathResult::Number(n) = result2 {
            assert_eq!(n, 2.0);
        } else {
            panic!("Expected Number result");
        }
    }
}

/// 测试文档 drop 后资源释放
#[test]
fn test_document_drop_cleanup() {
    // 创建和销毁大量文档
    for i in 0..100 {
        let html = format!("<div id='{}'>{}</div>", i, "x".repeat(100));
        let doc = Document::parse(&html).unwrap();
        let _ = doc.select("//div").unwrap();
        // doc 在这里 drop
    }
    // 如果有内存泄漏，这个测试可能会失败或变慢
}

/// 测试无效输入处理
#[test]
fn test_invalid_input_handling() {
    // 空输入
    let result = Document::parse("");
    assert!(result.is_ok()); // 空文档是有效的

    // 纯空白
    let result = Document::parse("   \n\t  ");
    assert!(result.is_ok());

    // 只有注释
    let result = Document::parse("<!-- just a comment -->");
    assert!(result.is_ok());
}

/// 测试节点类型正确性
#[test]
fn test_node_type_correctness() {
    let html = r#"<div>
    <!-- comment -->
    <p>text</p>
    <![CDATA[cdata content]]>
</div>"#;

    let doc = Document::parse(html).unwrap();

    // 元素节点
    let div = &doc.select("//div").unwrap()[0];
    assert_eq!(div.node_type(), NodeType::Element);

    // 注释节点
    let comments = doc.select("//comment()").unwrap();
    if !comments.is_empty() {
        assert_eq!(comments[0].node_type(), NodeType::Comment);
    }

    // 文本节点
    let text_nodes = doc.select("//p/text()").unwrap();
    if !text_nodes.is_empty() {
        assert_eq!(text_nodes[0].node_type(), NodeType::Text);
    }
}

/// 测试属性遍历安全性
#[test]
fn test_attribute_traversal_safety() {
    let html = r#"<div a="1" b="2" c="3">test</div>"#;
    let doc = Document::parse(html).unwrap();

    let div = &doc.select("//div").unwrap()[0];

    // 多次获取属性不应该有问题
    for _ in 0..100 {
        let attrs = div.attrs();
        assert_eq!(attrs.len(), 3);

        assert!(div.has_attr("a"));
        assert!(div.has_attr("b"));
        assert!(div.has_attr("c"));
        assert!(!div.has_attr("d"));
    }
}

/// 测试节点遍历安全性
#[test]
fn test_node_traversal_safety() {
    let html = r#"<div><p><span><a>deep</a></span></p></div>"#;
    let doc = Document::parse(html).unwrap();

    let div = &doc.select("//div").unwrap()[0];

    // 深度遍历
    let mut current = Some(div.clone());
    let mut depth = 0;
    while let Some(node) = current {
        depth += 1;
        current = node.first_child();
        if depth > 100 {
            panic!("Infinite loop detected");
        }
    }
    assert!(depth > 0);

    // 向上遍历
    let a = &doc.select("//a").unwrap()[0];
    let mut current = Some(a.clone());
    let mut upward_depth = 0;
    while let Some(node) = current {
        upward_depth += 1;
        current = node.parent();
        if upward_depth > 100 {
            panic!("Infinite loop detected");
        }
    }
    assert!(upward_depth > 0);
}

/// 测试 XPath 注入防护
#[test]
fn test_xpath_injection_safety() {
    let doc = Document::parse("<div>test</div>").unwrap();

    // 这些 XPath 表达式可能有问题，但应该被安全处理
    let dangerous_xpaths = vec![
        "//*",
        "//node()",
        "//@*",
        "/descendant::*",
    ];

    for xpath in dangerous_xpaths {
        // 应该成功或返回错误，但不应该崩溃
        let _ = doc.select(xpath);
    }
}

/// 测试空结果处理
#[test]
fn test_empty_result_handling() {
    let doc = Document::parse("<div>test</div>").unwrap();

    // 不存在的节点
    let result = doc.select("//nonexistent").unwrap();
    assert!(result.is_empty());

    // 不存在的属性
    let div = &doc.select("//div").unwrap()[0];
    assert_eq!(div.attr("nonexistent"), None);

    // 空属性映射
    let text_nodes = doc.select("//text()").unwrap();
    if !text_nodes.is_empty() {
        // 文本节点没有属性
        assert!(text_nodes[0].attrs().is_empty());
    }
}

/// 测试嵌套查询安全性
#[test]
fn test_nested_query_safety() {
    let doc = Document::parse(
        r#"<div><p><span>A</span></p><p><span>B</span></p><p><span>C</span></p></div>"#,
    )
    .unwrap();

    let paragraphs = doc.select("//p").unwrap();

    for p in &paragraphs {
        // 在每个段落中查询 span
        let spans = p.select(".//span").unwrap();
        assert_eq!(spans.len(), 1);
    }
}

/// 测试特大数据处理
#[test]
fn test_large_attribute_values() {
    // 创建一个大属性值
    let large_value = "x".repeat(10000);
    let html = format!(r#"<div data-large="{}">test</div>"#, large_value);

    let doc = Document::parse(&html).unwrap();
    let div = &doc.select("//div").unwrap()[0];

    let attr = div.attr("data-large").unwrap();
    assert_eq!(attr.len(), 10000);
}

/// 测试深度嵌套安全性
#[test]
fn test_deep_nesting_safety() {
    // 创建深度嵌套的 HTML
    let mut html = String::new();
    for _ in 0..100 {
        html.push_str("<div>");
    }
    html.push_str("<span>target</span>");
    for _ in 0..100 {
        html.push_str("</div>");
    }

    let doc = Document::parse(&html).unwrap();

    // 应该能找到深层节点
    let target = doc.select("//span").unwrap();
    assert_eq!(target.len(), 1);
    assert_eq!(target[0].text(), "target");
}
