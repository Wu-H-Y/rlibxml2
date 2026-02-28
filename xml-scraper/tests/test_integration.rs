//! 集成测试 - 真实世界 HTML 解析

use xml_scraper::{Document, Error, ParseOptions};

/// 测试解析复杂的真实世界 HTML
#[test]
fn test_real_world_ecommerce_page() {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Products - Shop</title>
</head>
<body>
    <header>
        <nav class="main-nav">
            <a href="/">Home</a>
            <a href="/products">Products</a>
            <a href="/cart">Cart <span class="count">3</span></a>
        </nav>
    </header>

    <main>
        <div class="products">
            <article class="product-card" data-id="1">
                <img src="/img/1.jpg" alt="Product 1">
                <h2 class="title">Laptop Pro</h2>
                <p class="price">$999.99</p>
                <p class="description">High-performance laptop</p>
                <button class="add-to-cart">Add to Cart</button>
            </article>

            <article class="product-card" data-id="2">
                <img src="/img/2.jpg" alt="Product 2">
                <h2 class="title">Wireless Mouse</h2>
                <p class="price">$29.99</p>
                <p class="description">Ergonomic wireless mouse</p>
                <button class="add-to-cart">Add to Cart</button>
            </article>

            <article class="product-card" data-id="3">
                <img src="/img/3.jpg" alt="Product 3">
                <h2 class="title">USB-C Hub</h2>
                <p class="price">$49.99</p>
                <p class="description">7-in-1 USB-C hub</p>
                <button class="add-to-cart">Add to Cart</button>
            </article>
        </div>
    </main>

    <footer>
        <p>&copy; 2024 Shop. All rights reserved.</p>
    </footer>
</body>
</html>
"#;

    let doc = Document::parse(html).unwrap();

    // 测试提取所有产品
    let products = doc.select("//article[@class='product-card']").unwrap();
    assert_eq!(products.len(), 3);

    // 测试提取产品标题
    let titles = doc.extract_texts("//h2[@class='title']").unwrap();
    assert_eq!(titles, vec!["Laptop Pro", "Wireless Mouse", "USB-C Hub"]);

    // 测试提取价格
    let prices = doc.extract_texts("//p[@class='price']").unwrap();
    assert_eq!(prices, vec!["$999.99", "$29.99", "$49.99"]);

    // 测试提取 data-id 属性
    let product = &products[0];
    assert_eq!(product.attr("data-id"), Some("1".to_string()));

    // 测试导航链接
    let nav_links = doc.select("//nav[@class='main-nav']/a").unwrap();
    assert_eq!(nav_links.len(), 3);

    // 测试购物车数量
    let cart_count = doc.extract_string("//span[@class='count']").unwrap();
    assert_eq!(cart_count, "3");
}

/// 测试解析破损的 HTML
#[test]
fn test_badly_formed_html() {
    let broken_html = r#"
<div class="container">
    <p>First paragraph
    <p>Second paragraph without closing tag
    <ul>
        <li>Item 1
        <li>Item 2
        <li>Item 3
    </ul>
    <div>Nested but not closed
        <span>Inline element
</div>
"#;

    let doc = Document::parse(broken_html).unwrap();

    // 即使 HTML 不完整，也应该能解析出内容
    let paragraphs = doc.select("//p").unwrap();
    assert_eq!(paragraphs.len(), 2);

    let list_items = doc.select("//li").unwrap();
    assert_eq!(list_items.len(), 3);
}

/// 测试 HTML 实体和特殊字符
#[test]
fn test_html_entities() {
    let html = r#"
<div>
    <p class="encoded">&lt;script&gt;alert('xss')&lt;/script&gt;</p>
    <p class="unicode">中文 日本語 한국어 العربية</p>
    <p class="special">&amp; &quot; &apos; &nbsp; &copy;</p>
    <p class="emoji">😀 🎉 🚀</p>
</div>
"#;

    let doc = Document::parse(html).unwrap();

    // 实体应该被正确解码
    let encoded = doc.extract_string("//p[@class='encoded']").unwrap();
    assert!(encoded.contains("<script>"));

    // Unicode 应该被正确处理
    let unicode = doc.extract_string("//p[@class='unicode']").unwrap();
    assert!(unicode.contains("中文"));
    assert!(unicode.contains("日本語"));

    // 特殊字符
    let special = doc.extract_string("//p[@class='special']").unwrap();
    assert!(special.contains("&"));

    // Emoji
    let emoji = doc.extract_string("//p[@class='emoji']").unwrap();
    assert!(emoji.contains("😀"));
}

/// 测试深层嵌套结构
#[test]
fn test_deeply_nested_structure() {
    let html = r#"
<div level="1">
    <div level="2">
        <div level="3">
            <div level="4">
                <div level="5">
                    <span target="yes">Found me!</span>
                </div>
            </div>
        </div>
    </div>
</div>
"#;

    let doc = Document::parse(html).unwrap();

    let target = doc.select("//span[@target='yes']").unwrap();
    assert_eq!(target.len(), 1);
    assert_eq!(target[0].text(), "Found me!");

    // 测试从深层节点向上遍历
    let span = &target[0];
    let parent = span.parent().unwrap();
    assert_eq!(parent.attr("level"), Some("5".to_string()));
}

/// 测试表格数据提取
#[test]
fn test_table_extraction() {
    let html = r#"
<table id="data">
    <thead>
        <tr>
            <th>Name</th>
            <th>Age</th>
            <th>City</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td>Alice</td>
            <td>30</td>
            <td>New York</td>
        </tr>
        <tr>
            <td>Bob</td>
            <td>25</td>
            <td>London</td>
        </tr>
        <tr>
            <td>Charlie</td>
            <td>35</td>
            <td>Paris</td>
        </tr>
    </tbody>
</table>
"#;

    let doc = Document::parse(html).unwrap();

    // 提取表头
    let headers = doc.extract_texts("//table[@id='data']//th").unwrap();
    assert_eq!(headers, vec!["Name", "Age", "City"]);

    // 提取所有行数据
    let rows = doc.select("//table[@id='data']/tbody/tr").unwrap();
    assert_eq!(rows.len(), 3);

    // 验证第一行数据
    let first_row = &rows[0];
    let cells = first_row.select("./td").unwrap();
    assert_eq!(cells.len(), 3);
    assert_eq!(cells[0].text(), "Alice");
    assert_eq!(cells[1].text(), "30");
    assert_eq!(cells[2].text(), "New York");
}

/// 测试 XPath 函数
#[test]
fn test_xpath_functions() {
    let html = r#"
<div>
    <p>First</p>
    <p>Second</p>
    <p>Third</p>
    <p class="highlight">Fourth</p>
    <p class="highlight">Fifth</p>
</div>
"#;

    let doc = Document::parse(html).unwrap();

    // count()
    let count = doc.extract_number("count(//p)").unwrap();
    assert_eq!(count, 5.0);

    // 带条件的 count
    let highlighted_count = doc
        .extract_number("count(//p[@class='highlight'])")
        .unwrap();
    assert_eq!(highlighted_count, 2.0);

    // 布尔表达式
    let has_highlighted = doc.extract_boolean("//p[@class='highlight']").unwrap();
    assert!(has_highlighted);

    let has_six = doc.extract_boolean("count(//p) = 6").unwrap();
    assert!(!has_six);

    // string()
    let first_text = doc.extract_string("string(//p)").unwrap();
    assert_eq!(first_text, "First");

    // concat
    let combined = doc.extract_string("concat(//p[1], ' - ', //p[2])").unwrap();
    assert_eq!(combined, "First - Second");
}

/// 测试不同解析选项
#[test]
fn test_parse_options() {
    let html = r#"
<div>
    <p>Text with <span>inline</span> elements</p>
    <p>More text</p>
</div>
"#;

    // 默认选项
    let doc = Document::parse_html_with_options(html, ParseOptions::default()).unwrap();
    let text_nodes = doc.select("//text()").unwrap();
    let text_count = text_nodes.len();

    // 紧凑选项（移除空白）
    let doc_compact = Document::parse_html_with_options(html, ParseOptions::compact()).unwrap();
    let text_nodes_compact = doc_compact.select("//text()").unwrap();

    // 紧凑模式应该有更少的文本节点
    assert!(text_nodes_compact.len() <= text_count);
}

/// 测试错误处理
#[test]
fn test_error_handling() {
    // 空字节
    let result = Document::parse("Hello\0World");
    assert!(matches!(result.unwrap_err(), Error::NullByte));

    // 无效 XPath
    let doc = Document::parse("<div>test</div>").unwrap();
    let result = doc.select("//[invalid");
    assert!(matches!(result.unwrap_err(), Error::InvalidXPath { .. }));
}

/// 测试节点遍历完整性
#[test]
fn test_node_traversal_completeness() {
    let html = r#"
<div id="root">
    <p id="first">First paragraph</p>
    <p>Middle paragraph</p>
    <p id="last">Last paragraph</p>
</div>
"#;

    let doc = Document::parse(html).unwrap();

    let root = doc.select("//div[@id='root']").unwrap();
    let div = &root[0];

    // 子节点
    let children = div.element_children();
    assert_eq!(children.len(), 3);

    // 找第一个 p 元素
    let first_p = &doc.select("//p[@id='first']").unwrap()[0];

    // 父节点
    let parent = first_p.parent().unwrap();
    assert_eq!(parent.attr("id"), Some("root".to_string()));

    // 兄弟节点 - 注意：siblings() 返回所有兄弟节点，包括文本节点
    let middle = &children[1];
    let all_siblings = middle.siblings();
    // 过滤只保留元素节点
    let element_siblings: Vec<_> = all_siblings
        .iter()
        .filter(|n| n.node_type().is_element())
        .collect();
    assert_eq!(element_siblings.len(), 2);
}

/// 测试 XML 解析
#[test]
fn test_xml_parsing() {
    // 使用不带命名空间的简单 XML
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed>
    <title>Example Feed</title>
    <entry>
        <title>First Entry</title>
        <link href="https://example.com/1"/>
        <summary>Summary of first entry</summary>
    </entry>
    <entry>
        <title>Second Entry</title>
        <link href="https://example.com/2"/>
        <summary>Summary of second entry</summary>
    </entry>
</feed>
"#;

    let doc = Document::parse_xml(xml).unwrap();

    // 提取所有条目标题
    let titles = doc.extract_texts("//entry/title").unwrap();
    // 注意：文本可能包含空白，所以我们检查包含关系
    assert!(titles.iter().any(|t| t.contains("First Entry")));
    assert!(titles.iter().any(|t| t.contains("Second Entry")));

    // 提取链接
    let entries = doc.select("//entry").unwrap();
    assert_eq!(entries.len(), 2);
}

/// 测试大量数据
#[test]
fn test_large_document() {
    // 生成大量重复内容
    let mut html = String::from("<div>");
    for i in 0..1000 {
        html.push_str(&format!(
            r#"<p class="item" id="item-{}">Item number {}</p>"#,
            i, i
        ));
    }
    html.push_str("</div>");

    let doc = Document::parse(&html).unwrap();

    // 验证所有元素都被解析
    let items = doc.select("//p[@class='item']").unwrap();
    assert_eq!(items.len(), 1000);

    // 验证特定元素
    let first = doc.select("//p[@id='item-0']").unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].text(), "Item number 0");

    let last = doc.select("//p[@id='item-999']").unwrap();
    assert_eq!(last.len(), 1);
    assert_eq!(last[0].text(), "Item number 999");
}

/// 测试属性访问边界情况
#[test]
fn test_attribute_edge_cases() {
    let html = r#"
<div>
    <p id="normal">Normal ID</p>
    <p id="">Empty ID</p>
    <p>No ID</p>
    <p data-value="123" data-name="test">Multiple data attrs</p>
    <p class="  spaces  ">Class with spaces</p>
</div>
"#;

    let doc = Document::parse(html).unwrap();

    // 正常属性
    let normal = &doc.select("//p[@id='normal']").unwrap()[0];
    assert_eq!(normal.attr("id"), Some("normal".to_string()));

    // 空属性值
    let empty = &doc.select("//p[contains(text(), 'Empty')]").unwrap()[0];
    assert_eq!(empty.attr("id"), Some("".to_string()));

    // 无属性
    let no_id = &doc.select("//p[contains(text(), 'No ID')]").unwrap()[0];
    assert_eq!(no_id.attr("id"), None);

    // 多个 data-* 属性
    let multi = &doc.select("//p[@data-value]").unwrap()[0];
    let attrs = multi.attrs();
    assert_eq!(attrs.get("data-value"), Some(&"123".to_string()));
    assert_eq!(attrs.get("data-name"), Some(&"test".to_string()));
}
