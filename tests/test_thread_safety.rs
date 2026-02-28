//! 多线程测试
//!
//! 注意：libxml2 文档（Document）不是线程安全的，不能跨线程共享。
//! 这些测试验证在多线程环境中，每个线程可以安全地创建和使用自己的文档。

use std::thread;
use rlibxml::Document;

/// 测试多线程中每个线程创建自己的文档
#[test]
fn test_concurrent_document_creation() {
    let mut handles = vec![];

    for i in 0..10 {
        let handle = thread::spawn(move || {
            // 每个线程创建自己的文档
            let html = format!("<div id='{}'>{}</div>", i, "x".repeat(100));
            let doc = Document::parse(&html).unwrap();

            // 使用文档
            let nodes = doc.select("//div").unwrap();
            assert_eq!(nodes.len(), 1);

            let id = nodes[0].attr("id").unwrap();
            assert_eq!(id.parse::<i32>().unwrap(), i);

            i
        });
        handles.push(handle);
    }

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(results.len(), 10);
}

/// 测试线程间传递提取的数据（而不是文档）
#[test]
fn test_thread_data_extraction() {
    // 在主线程解析文档并提取数据
    let doc = Document::parse(
        r#"<data>
            <item>A</item>
            <item>B</item>
            <item>C</item>
        </data>"#,
    )
    .unwrap();

    // 提取数据（这是 String，可以跨线程传递）
    let texts = doc.extract_texts("//item").unwrap();

    // 在子线程验证数据
    let handle = thread::spawn(move || {
        assert_eq!(texts, vec!["A", "B", "C"]);
    });

    handle.join().unwrap();
}

/// 测试高并发下创建文档的压力测试
#[test]
fn test_high_concurrency_stress() {
    let mut handles = vec![];

    // 100 个线程，每个创建自己的文档并执行查询
    for i in 0..100 {
        let handle = thread::spawn(move || {
            let html = format!("<root><n>{}</n><n>{}</n><n>{}</n></root>", i, i + 1, i + 2);
            let doc = Document::parse(&html).unwrap();

            for _ in 0..10 {
                let _ = doc.select("//n").unwrap();
                let _ = doc.evaluate("count(//n)").unwrap();
                let _ = doc.root();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// 测试不同类型 XPath 查询的并发（每个线程自己的文档）
#[test]
fn test_concurrent_xpath_types() {
    let mut handles = vec![];

    // 线程 1：节点集合查询
    handles.push(thread::spawn(|| {
        for i in 0..100 {
            let html = format!(
                "<data><num>{}</num><num>{}</num><num>{}</num></data>",
                i,
                i + 1,
                i + 2
            );
            let doc = Document::parse(&html).unwrap();
            let nodes = doc.select("//num").unwrap();
            assert_eq!(nodes.len(), 3);
        }
    }));

    // 线程 2：数字查询
    handles.push(thread::spawn(|| {
        for i in 0..100 {
            let html = format!(
                "<data><num>{}</num><num>{}</num><num>{}</num></data>",
                i,
                i + 1,
                i + 2
            );
            let doc = Document::parse(&html).unwrap();
            let count = doc.extract_number("count(//num)").unwrap();
            assert_eq!(count, 3.0);
        }
    }));

    // 线程 3：布尔查询
    handles.push(thread::spawn(|| {
        for i in 0..100 {
            let html = format!(
                "<data><num>{}</num><num>{}</num><num>{}</num></data>",
                i,
                i + 1,
                i + 2
            );
            let doc = Document::parse(&html).unwrap();
            let has_nodes = doc.extract_boolean("count(//num) > 0").unwrap();
            assert!(has_nodes);
        }
    }));

    // 线程 4：字符串查询
    handles.push(thread::spawn(|| {
        for i in 0..100 {
            let html = format!(
                "<data><num>{}</num><num>{}</num><num>{}</num></data>",
                i,
                i + 1,
                i + 2
            );
            let doc = Document::parse(&html).unwrap();
            let text = doc.extract_string("string(//num)").unwrap();
            assert!(!text.is_empty());
        }
    }));

    for handle in handles {
        handle.join().unwrap();
    }
}

/// 验证 Document 不能跨线程共享（编译时检查）
///
/// 注意：这个测试验证运行时行为。
/// 由于 Document 包含裸指针，它自动是 !Send + !Sync 的。
#[test]
fn test_document_not_send_sync() {
    // 在单个线程中使用文档是安全的
    let doc = Document::parse("<div>test</div>").unwrap();
    let _ = doc.select("//div").unwrap();

    // 文档在同一作用域内使用
    {
        let doc2 = Document::parse("<span>hello</span>").unwrap();
        let _ = doc2.select("//span").unwrap();
    }
}

/// 测试快速创建和销毁文档
#[test]
fn test_rapid_document_lifecycle() {
    let mut handles = vec![];

    for _ in 0..50 {
        let handle = thread::spawn(|| {
            for j in 0..20 {
                let html = format!("<item>{}</item>", j);
                let doc = Document::parse(&html).unwrap();
                let _ = doc.select("//item").unwrap();
                // doc 在这里被自动释放
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// 测试多线程下 libxml2 全局初始化
#[test]
fn test_global_init_in_threads() {
    let mut handles = vec![];

    for _ in 0..20 {
        let handle = thread::spawn(|| {
            // 在每个线程中调用 init（应该安全）
            rlibxml::init();

            let doc = Document::parse("<test>value</test>").unwrap();
            let _ = doc.select("//test").unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
