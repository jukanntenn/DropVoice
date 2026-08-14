//! OpenAPI 文档漂移检测：重新生成并与提交的 `docs/openapi.{json,yaml}` 字节比较。
//!
//! 任何修改 API 面（handler 注解、domain schema、错误组件等）而忘记重新生成文档，
//! 本测试都会失败，并提示运行：
//!
//! ```sh
//! cargo run -p dropvoice-pairing-server --bin gen-openapi
//! ```
//!
//! prek 提交钩子在 `apps/pairing-server/` 文件变更时自动触发本测试。

use std::path::Path;

fn manifest_dir() -> &'static str {
    env!("CARGO_MANIFEST_DIR")
}

fn read_committed(name: &str) -> String {
    let path = Path::new(manifest_dir()).join("docs").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "missing {path:?}: run `cargo run -p dropvoice-pairing-server --bin gen-openapi` first"
        )
    })
}

#[test]
fn openapi_docs_are_up_to_date() {
    let (json, yaml) = dropvoice_pairing_server::docs::generate_specs();

    let json_on_disk = read_committed("openapi.json");
    let yaml_on_disk = read_committed("openapi.yaml");

    assert_eq!(
        json, json_on_disk,
        "docs/openapi.json is out of date: run `cargo run -p dropvoice-pairing-server --bin gen-openapi` and commit the regenerated files"
    );
    assert_eq!(
        yaml, yaml_on_disk,
        "docs/openapi.yaml is out of date: run `cargo run -p dropvoice-pairing-server --bin gen-openapi` and commit the regenerated files"
    );
}
