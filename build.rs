use std::{env, fs};
use std::path::Path;
use std::process::Command;

use yaml_rust::{YamlEmitter, YamlLoader};

fn main() {
    // OpenAPI仕様ファイルの変更を検知して再ビルドする
    println!("cargo:rerun-if-changed=openapi.yaml");

    // Cargoプロジェクトのルートディレクトリを取得
    let project_dir = {
        let str = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        Path::new(&str).to_path_buf()
    };

    // OpenAPI仕様ファイルと出力ディレクトリのパスを構築
    let openapi_spec = project_dir.join("openapi.yaml");
    let output_dir = project_dir.join("openapi_gen");

    fix_float_and_double_format(&openapi_spec);
    openapi_gen(&openapi_spec, &output_dir);
}

fn fix_float_and_double_format(path: &Path) {
    let content = fs::read_to_string(path).expect("Failed to read openapi.yaml");

    let mut docs = YamlLoader::load_from_str(&content).expect("Failed to parse YAML");

    // パースされたYAMLから最初のドキュメントを取得
    let doc = docs.get_mut(0).expect("No document found in YAML");

    // 再帰的にYAMLを探索し、floatやdoubleの制約値を修正
    fix_float_and_double_limits(doc);

    // 修正されたYAMLを文字列に戻す
    let mut modified_content = String::new();
    let mut emitter = YamlEmitter::new(&mut modified_content);
    emitter.dump(doc).expect("Failed to emit YAML");

    // 修正された内容でファイルを上書き保存
    fs::write(path, modified_content).expect("Failed to write updated openapi.yaml");
}

// 再帰的にYAMLを探索して、floatやdoubleの制約値を修正する関数
fn fix_float_and_double_limits(yaml: &mut yaml_rust::Yaml) {
    match yaml {
        yaml_rust::Yaml::Hash(hash) => {
            for (key, value) in hash.iter_mut() {
                if let yaml_rust::Yaml::String(key_str) = key {
                    if key_str == "minimum" || key_str == "maximum" {
                        if let yaml_rust::Yaml::Integer(num) = value {
                            // 値が整数の場合、小数形式に変換
                            *value = yaml_rust::Yaml::Real(format!("{}.0", num));
                        }
                    }
                }
                // 再帰的にネストされたYAMLを探索
                fix_float_and_double_limits(value);
            }
        }
        yaml_rust::Yaml::Array(arr) => {
            for item in arr.iter_mut() {
                fix_float_and_double_limits(item);
            }
        }
        _ => {}
    }
}

// OpenAPI Generatorを使ってAxumのコードを生成する関数
fn openapi_gen(openapi_spec: &Path, output_dir: &Path) {
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
}
