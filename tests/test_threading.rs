use rlibxml::Document;
use std::sync::Arc;
use std::thread;

const HTML: &str = r#"
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

#[test]
fn test_document_send() {
    let doc = Document::parse(HTML).unwrap();

    // 验证 Document 可以 Send 到另一个线程
    let handle = thread::spawn(move || {
        let items = doc.extract_texts("//li").unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], "Apple");
    });

    handle.join().unwrap();
}

#[test]
fn test_document_sync() {
    let doc = Arc::new(Document::parse(HTML).unwrap());

    let mut handles = vec![];

    // 验证 Document 可以通过 Arc 在多个线程之间 Sync (共享不可变借用)
    for i in 0..4 {
        let doc_clone = Arc::clone(&doc);
        let handle = thread::spawn(move || {
            let count = doc_clone.extract_number("count(//li)").unwrap();
            assert_eq!(count, 3.0);

            // 简单验证 XPath
            let string_val = doc_clone
                .extract_string(&format!("string(//li[{}])", i % 3 + 1))
                .unwrap();
            assert!(!string_val.is_empty());
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_selected_node_send() {
    let doc = Document::parse(HTML).unwrap();
    let body_nodes = doc.select("//body").unwrap();
    let body_node = &body_nodes[0];

    // 验证带有生命周期的 SelectedNode 也能在作用域允许的情况下 Send
    // std::thread::scope (Rust 1.63+) 是验证生命周期边界跨线程的标准方式
    thread::scope(|s| {
        s.spawn(|| {
            let items = body_node.select(".//li").unwrap();
            assert_eq!(items.len(), 3);
        });
    });
}
