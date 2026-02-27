// wrapper.h - 用于 bindgen 生成 Rust FFI 绑定
// 仅包含爬虫所需的 HTML 解析和 XPath 功能

#include <libxml/HTMLparser.h>
#include <libxml/xpath.h>
#include <libxml/xpathInternals.h>
