//! 导出 OpenAPI 3.1 文档到 `docs/openapi.{json,yaml}`。
//!
//! 用法：
//! ```sh
//! cargo run -p dropvoice-pairing-server --bin gen-openapi
//! ```
//!
//! 生成后提交 `docs/openapi.json` 与 `docs/openapi.yaml`；
//! `tests/openapi.rs` 与 prek hook 会校验两者与代码无漂移。

use std::path::Path;

fn main() -> anyhow::Result<()> {
    let (json, yaml) = dropvoice_pairing_server::docs::generate_specs();

    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs");
    std::fs::create_dir_all(&dir)?;
    let json_path = dir.join("openapi.json");
    let yaml_path = dir.join("openapi.yaml");
    std::fs::write(&json_path, json)?;
    std::fs::write(&yaml_path, yaml)?;

    eprintln!("Generated: {}", json_path.display());
    eprintln!("Generated: {}", yaml_path.display());
    Ok(())
}
