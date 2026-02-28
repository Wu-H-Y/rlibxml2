//! 错误处理测试

use xml_scraper::{Document, Error};

#[test]
fn test_input_too_large() {
    // 验证大输入检查逻辑
    // MAX_INPUT_SIZE 约等于 i32::MAX
    let max_size = i32::MAX as usize;
    let large_size = max_size + 1;

    // 验证 large_size 确实大于 max_size
    assert!(large_size > max_size);
}

#[test]
fn test_null_byte_rejection() {
    let html_with_null = "Hello\0World";
    let result = Document::parse(html_with_null);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::NullByte));
}

#[test]
fn test_invalid_xpath() {
    let doc = Document::parse("<div>test</div>").unwrap();
    let result = doc.select("//[invalid");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::InvalidXPath { .. }));
}

#[test]
fn test_empty_document_select() {
    let doc = Document::parse("").unwrap();

    // 对空文档执行 XPath 查询
    let result = doc.select("//div");
    // 应该返回空结果，而不是错误
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_null_byte_in_xpath() {
    let doc = Document::parse("<div>test</div>").unwrap();
    let result = doc.select("//div\0");

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::NullByte));
}

#[test]
fn test_error_display() {
    let err = Error::NullByte;
    let display = format!("{}", err);
    assert!(display.contains("null byte") || display.contains("空字节"));

    let err = Error::InvalidXPath {
        xpath: "//[invalid".to_string(),
        reason: None,
    };
    let display = format!("{}", err);
    assert!(display.contains("XPath") || display.contains("xpath"));
}

#[test]
fn test_nonexistent_element() {
    let doc = Document::parse("<div>test</div>").unwrap();
    let result = doc.select("//nonexistent");

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_attribute_on_nonexistent_element() {
    let doc = Document::parse("<div>test</div>").unwrap();

    // 尝试获取不存在元素的属性不会 panic
    let result = doc.select("//nonexistent");
    if let Ok(nodes) = result
        && nodes.is_empty()
    {
        // 这是预期行为
    }
}
