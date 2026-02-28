# rlibxml2

[![CI](https://github.com/Wu-H-Y/rlibxml2/workflows/CI/badge.svg)](https://github.com/Wu-H-Y/rlibxml2/actions)

安全的 Rust HTML/XML 解析与 XPath 查询库，基于 [libxml2](https://gitlab.gnome.org/GNOME/libxml2) 封装。

## 特性

- **零外部依赖**：无需系统安装 libxml2，源码编译静态链接
- **移动端友好**：精简编译配置，避免交叉编译地狱
- **内存安全**：通过生命周期绑定确保节点不会超出文档生命周期
- **安全封装**：所有 unsafe 代码被隔离在内部模块，公开 API 100% 安全
- **容错解析**：专为处理真实世界的脏 HTML 设计
- **跨平台**：支持 Windows / macOS / Linux / Android / iOS

## 安装

```toml
[dependencies]
xml-scraper = "0.1"
```

## 快速开始

```rust
use xml_scraper::Document;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let doc = Document::parse(html)?;

    // 方式 1: 直接提取文本
    let items = doc.extract_texts("//li[@class='item']")?;
    println!("Items: {:?}", items); // ["Apple", "Banana", "Cherry"]

    // 方式 2: 获取节点进行更多操作
    let nodes = doc.select("//li")?;
    for node in nodes {
        println!("Tag: {}, Text: {}", node.tag_name(), node.text());
        println!("Path: {}", node.path());
    }

    Ok(())
}
```

## 生命周期安全

节点引用的生命周期绑定到所属的文档，Rust 编译器会在编译时阻止悬垂引用：

```rust,compile_fail
use xml_scraper::Document;

let node = {
    let doc = Document::parse("<div>test</div>").unwrap();
    doc.select("//div").unwrap()[0].clone()
    // doc 在此处被 drop
};
node.text();  // 编译错误：`doc` 的生命周期不够长
```

## 容错解析

真实世界的 HTML 往往是破损的。本库默认开启最大容错模式：

```rust
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

let doc = Document::parse(broken_html)?;
let items = doc.extract_texts("//li")?;
assert_eq!(items, vec!["Item 1", "Item 2"]);
```

## 自定义解析选项

```rust
use xml_scraper::{Document, ParseOptions};

let options = ParseOptions {
    recover: true,      // 容错模式
    no_error: true,     // 抑制错误
    no_warning: true,   // 抑制警告
    no_blanks: true,    // 移除空白节点
};

let doc = Document::parse_html_with_options(html, options)?;
```

## XPath 查询

支持完整的 XPath 1.0 语法：

```rust
use xml_scraper::{Document, XPathResult};

let doc = Document::parse("<div><p>A</p><p>B</p></div>")?;

// 节点集合
let nodes = doc.select("//p")?;

// 数字
let count = doc.extract_number("count(//p)")?;  // 2.0

// 布尔值
let has_multiple = doc.extract_boolean("count(//p) > 1")?;  // true

// 字符串
let first = doc.extract_string("string(//p)")?;  // "A"

// 任意类型
let result = doc.evaluate("count(//p)")?;
if let XPathResult::Number(n) = result {
    println!("Count: {}", n);
}
```

## 节点遍历

```rust
use xml_scraper::Document;

let doc = Document::parse("<div><p>A</p><p>B</p></div>")?;
let div = &doc.select("//div")?[0];

// 子节点
let children = div.children();
let element_children = div.element_children();

// 父节点
let parent = div.parent();

// 兄弟节点
let next = div.next_sibling();
let prev = div.prev_sibling();
let siblings = div.siblings();

// 属性访问
let attr = div.attr("class");
let has_class = div.has_attr("class");
let all_attrs = div.attrs();

// HTML 序列化
let inner = div.inner_html();
let outer = div.outer_html();
```

## 构建要求

- Rust 1.70+
- CMake 3.15+
- C 编译器 (GCC / Clang / MSVC)

### Windows

需要安装 Visual Studio Build Tools 和 CMake。

### Android 交叉编译

```bash
# 安装 Android target 和 cargo-ndk
rustup target add aarch64-linux-android
cargo install cargo-ndk

# 设置 NDK 路径
export ANDROID_NDK_HOME=/path/to/ndk

# 编译
cargo ndk -t arm64-v8a build --release
```

### iOS 交叉编译

```bash
# 安装 iOS targets
rustup target add aarch64-apple-ios
rustup target add aarch64-apple-ios-sim

# 编译真机版本
cargo build --target aarch64-apple-ios --release

# 编译模拟器版本
cargo build --target aarch64-apple-ios-sim --release
```

## 项目结构

```
rlibxml2/
├── Cargo.toml              # 工作区配置
├── libxml2-sys/            # 底层 FFI 绑定
│   ├── Cargo.toml
│   ├── build.rs            # CMake 构建脚本
│   ├── wrapper.h           # bindgen 头文件
│   ├── src/
│   │   ├── lib.rs          # 原始 FFI 导出
│   │   └── safe_wrapper.rs # 安全封装层
│   └── libxml2_src/        # libxml2 C 源码 (submodule)
└── xml-scraper/            # 安全 Rust 封装
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs          # 主要 API 导出
    │   ├── document.rs     # 文档解析
    │   ├── node.rs         # 节点操作
    │   ├── xpath.rs        # XPath 查询
    │   ├── options.rs      # 解析选项
    │   ├── node_type.rs    # 节点类型
    │   └── error.rs        # 错误类型
    └── tests/              # 测试用例
```

## License

MIT
