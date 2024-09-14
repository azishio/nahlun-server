use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    openapi_gen();
}

fn openapi_gen() {
    // Cargoプロジェクトのルートディレクトリを取得
    let project_dir = {
        let str = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        Path::new(&str).to_path_buf()
    };

    // OpenAPI仕様ファイルと出力ディレクトリのパスを構築
    let openapi_spec = project_dir.join("openapi.yaml");
    let output_dir = project_dir.join("openapi_gen");

    // コマンド生成と実行
    let status = Command::new("openapi-generator")
        .arg("generate")
        .args([
            "-i",
            openapi_spec
                .to_str()
                .expect("Failed to convert openapi.yaml path to str"),
            "-g",
            "rust-axum",
            "-o",
            output_dir
                .to_str()
                .expect("Failed to convert openapi_gen path to str"),
        ])
        .status()
        .expect("Failed to run openapi-generator");

    // コマンドが正常に終了したかをチェック
    if !status.success() {
        panic!("OpenAPI Generator failed to generate the code");
    }

    // OpenAPI仕様ファイルの変更を検知して再ビルドする
    println!("cargo:rerun-if-changed=openapi.yaml");
}
