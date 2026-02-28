use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let libxml2_src = manifest_dir.join("libxml2_src");

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=libxml2_src/");

    // 检查 libxml2 源码是否存在
    if !libxml2_src.exists() {
        panic!(
            "libxml2 source not found! Please run: git submodule update --init --recursive\n\
             Or manually clone: git clone https://gitlab.gnome.org/GNOME/libxml2.git libxml2-sys/libxml2_src"
        );
    }

    // 使用 cmake 编译极限精简版的 libxml2
    let mut cmake_config = cmake::Config::new(&libxml2_src);

    cmake_config
        .define("LIBXML2_WITH_TREE", "ON")
        .define("LIBXML2_WITH_HTML", "ON")
        .define("LIBXML2_WITH_XPATH", "ON")
        .define("LIBXML2_WITH_THREADS", "ON");

    // 关键：关闭移动端极易报错的额外依赖
    cmake_config
        .define("LIBXML2_WITH_ICONV", "OFF")
        .define("LIBXML2_WITH_ICU", "OFF")
        .define("LIBXML2_WITH_LZMA", "OFF")
        .define("LIBXML2_WITH_ZLIB", "OFF");

    // 砍掉不需要的模块
    cmake_config
        .define("LIBXML2_WITH_HTTP", "OFF")
        .define("LIBXML2_WITH_FTP", "OFF")
        .define("LIBXML2_WITH_PYTHON", "OFF")
        .define("LIBXML2_WITH_PROGRAMS", "OFF")
        .define("LIBXML2_WITH_TESTS", "OFF")
        .define("LIBXML2_WITH_VALID", "OFF")
        .define("LIBXML2_WITH_SCHEMAS", "OFF")
        .define("LIBXML2_WITH_CATALOG", "OFF")
        .define("LIBXML2_WITH_MEM_DEBUG", "OFF")
        .define("LIBXML2_WITH_DEBUG", "OFF")
        .define("LIBXML2_WITH_ISO8859X", "OFF")
        .define("LIBXML2_WITH_SAX1", "OFF");

    // 必须静态链接
    cmake_config.define("BUILD_SHARED_LIBS", "OFF");

    // Windows 特定配置
    if target.contains("windows") {
        cmake_config.define("CMAKE_WINDOWS_EXPORT_ALL_SYMBOLS", "ON");
    }

    let dst = cmake_config.build();

    // 设置库搜索路径和链接
    // cmake 输出固定为 lib/ 目录
    let lib_dir = dst.join("lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    // 库名规则:
    // - Windows MSVC 静态库: libxml2s (release) / libxml2sd (debug)
    // - 其他平台: xml2 (无后缀)
    let lib_name = if target.contains("windows") {
        if cfg!(debug_assertions) {
            "libxml2sd"
        } else {
            "libxml2s"
        }
    } else {
        "xml2"
    };
    println!("cargo:rustc-link-lib=static={}", lib_name);

    // 额外的系统库链接
    if target.contains("windows") {
        println!("cargo:rustc-link-lib=ws2_32");
        println!("cargo:rustc-link-lib=bcrypt"); // Windows 加密随机数 API
    } else if target.contains("android") {
        println!("cargo:rustc-link-lib=m");
    } else {
        println!("cargo:rustc-link-lib=pthread");
        println!("cargo:rustc-link-lib=dl");
    }

    // 生成 FFI 绑定 - cmake 输出目录包含所有头文件（含生成的 xmlversion.h）
    let include_dir = dst.join("include").join("libxml2");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", include_dir.display()))

        // ========================================
        // HTML 解析函数
        // ========================================
        .allowlist_function("htmlReadMemory")

        // ========================================
        // XML 文档函数
        // ========================================
        .allowlist_function("xmlFreeDoc")
        .allowlist_function("xmlNodeGetContent")
        .allowlist_function("xmlGetNodePath")
        .allowlist_function("xmlGetProp")
        .allowlist_function("xmlDocGetRootElement")
        .allowlist_function("xmlReadMemory")
        .allowlist_function("xmlCleanupParser")
        .allowlist_function("xmlInitParser")

        // ========================================
        // XPath 核心函数
        // ========================================
        .allowlist_function("xmlXPathNewContext")
        .allowlist_function("xmlXPathFreeContext")
        .allowlist_function("xmlXPathEvalExpression")
        .allowlist_function("xmlXPathFreeObject")
        .allowlist_function("xmlXPathSetContextNode")

        // ========================================
        // 核心类型
        // ========================================
        .allowlist_type("xmlDoc")
        .allowlist_type("xmlNode")
        .allowlist_type("xmlAttr")
        .allowlist_type("xmlNs")
        .allowlist_type("xmlDtd")
        .allowlist_type("xmlDict")
        .allowlist_type("xmlError")
        .allowlist_type("xmlXPathContext")
        .allowlist_type("xmlXPathObject")
        .allowlist_type("xmlXPathObjectPtr")
        .allowlist_type("xmlNodeSet")
        .allowlist_type("xmlNodeSetPtr")
        .allowlist_type("htmlParserOption")
        .allowlist_type("xmlParserOption")
        .allowlist_type("xmlElementType")
        .allowlist_type("xmlXPathVariable")
        .allowlist_type("xmlXPathFunct")

        // ========================================
        // 枚举值
        // ========================================
        .allowlist_var("htmlParserOption_.*")
        .allowlist_var("xmlParserOption_.*")
        .allowlist_var("xmlElementType_.*")
        .allowlist_var("XPATH_.*")
        .allowlist_var("XPTR_.*")
        .allowlist_var("XML_XPATH_.*")

        // ========================================
        // 全局变量
        // ========================================
        .allowlist_var("xmlFree")
        .allowlist_var("xmlMalloc")
        .allowlist_var("xmlRealloc")

        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
