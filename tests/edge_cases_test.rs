use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

/// エッジケース: 非常に長い変数名
#[test]
fn test_very_long_variable_name() {
    let dir = tempdir().unwrap();

    let long_name = "a".repeat(1000);
    let data = dir.path().join("data.yaml");
    fs::write(&data, format!("{}: value", long_name)).unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, format!("{{{{ {} }}}}", long_name)).unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("value\n");
}

/// エッジケース: 非常に長い変数値
#[test]
fn test_very_long_variable_value() {
    let dir = tempdir().unwrap();

    let long_value = "x".repeat(10000);
    let data = dir.path().join("data.yaml");
    fs::write(&data, format!("var: \"{}\"", long_value)).unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ var }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout(format!("{}\n", long_value));
}

/// エッジケース: 空白のみの変数名
#[test]
fn test_whitespace_only_variable() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "var: value").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{    }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success();
}

/// エッジケース: ドットのみの変数パス
#[test]
fn test_dot_only_path() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "a: 1").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ . }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success();
}

/// エッジケース: 連続したドット
#[test]
fn test_consecutive_dots() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "a:\n  b: value").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ a..b }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success();
}

/// エッジケース: 配列の範囲外アクセス
#[test]
fn test_array_out_of_bounds() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "items:\n  - one\n  - two").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ items.10 }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("\n"); // 空文字が期待される
}

/// エッジケース: 負の配列インデックス
#[test]
fn test_negative_array_index() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "items:\n  - one").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ items.-1 }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success();
}

/// エッジケース: 改行を含む変数値
#[test]
fn test_multiline_variable_value() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "text: |\n  line1\n  line2\n  line3").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ text }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout(predicate::str::contains("line1"))
        .stdout(predicate::str::contains("line2"));
}

/// エッジケース: タブ文字を含む変数値
#[test]
fn test_tab_characters() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "text: \"tab\there\"").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ text }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("tab\there\n");
}

/// エッジケース: 波括弧を含む変数値
#[test]
fn test_braces_in_value() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, r#"text: "has { and } braces""#).unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ text }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("has { and } braces\n");
}

/// エッジケース: 連続したエスケープ
#[test]
fn test_consecutive_escapes() {
    let dir = tempdir().unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, r"\{{ var1 }} \{{ var2 }} \{{ var3 }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .assert()
        .success()
        .stdout("{{ var1 }} {{ var2 }} {{ var3 }}\n");
}

/// エッジケース: エスケープと変数の混在
#[test]
fn test_mixed_escape_and_variables() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "real: value").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, r"{{ real }} and \{{ fake }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("value and {{ fake }}\n");
}

/// エッジケース: Include内のInclude内のInclude (深い階層)
#[test]
fn test_deeply_nested_includes() {
    let dir = tempdir().unwrap();

    // 10階層のincludeチェーン
    for i in 0..10 {
        let file = dir.path().join(format!("level{}.txt", i));
        if i < 9 {
            fs::write(&file, format!("L{} {{{{> level{}.txt }}}}", i, i + 1)).unwrap();
        } else {
            fs::write(&file, "L9 END").unwrap();
        }
    }

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(dir.path().join("level0.txt"))
        .assert()
        .success()
        .stdout(predicate::str::contains("L0"))
        .stdout(predicate::str::contains("END"));
}

/// エッジケース: 同じファイルを複数回include
#[test]
fn test_same_file_multiple_includes() {
    let dir = tempdir().unwrap();

    let partial = dir.path().join("partial.txt");
    fs::write(&partial, "{{ value }}").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{> partial.txt }}, {{> partial.txt }}, {{> partial.txt }}").unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "value: X").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("X, X, X\n");
}

/// エッジケース: 空のincludeファイル
#[test]
fn test_empty_include_file() {
    let dir = tempdir().unwrap();

    let included = dir.path().join("empty.txt");
    fs::write(&included, "").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "Before{{> empty.txt }}After").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .assert()
        .success()
        .stdout("BeforeAfter\n");
}

/// エッジケース: Includeパスに空白
#[test]
fn test_include_path_with_spaces() {
    let dir = tempdir().unwrap();

    let included = dir.path().join("file with spaces.txt");
    fs::write(&included, "content").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{> file with spaces.txt }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .assert()
        .success()
        .stdout("content\n");
}

/// エッジケース: 非常に大きなデータファイル
#[test]
fn test_large_data_file() {
    let dir = tempdir().unwrap();

    let mut data_content = String::from("vars:\n");
    for i in 0..1000 {
        data_content.push_str(&format!("  var{}: value{}\n", i, i));
    }

    let data = dir.path().join("data.yaml");
    fs::write(&data, data_content).unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ vars.var0 }}, {{ vars.var500 }}, {{ vars.var999 }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("value0, value500, value999\n");
}

/// エッジケース: 深いネストのオブジェクト
#[test]
fn test_deeply_nested_object() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(
        &data,
        "a:\n  b:\n    c:\n      d:\n        e:\n          f:\n            g: value",
    )
    .unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ a.b.c.d.e.f.g }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("value\n");
}

/// エッジケース: null値の扱い
#[test]
fn test_null_values() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "nullval: null\nemptyval: \"\"\nzero: 0\nfalse: false").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(
        &template,
        "[{{ nullval }}][{{ emptyval }}][{{ zero }}][{{ false }}]",
    )
    .unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("[][][0][false]\n");
}

// YAMLのアンカーとエイリアス（<<: *anchor）はYAML 1.1の機能で、
// YAML 1.2では非推奨。serde_yaml 0.9はYAML 1.2ベースのため、
// マージキーは正しくサポートされない。これはライブラリの制限。
// （data_merge_test.rsのコメントも参照）

/// エッジケース: 数値のみの変数名
#[test]
fn test_numeric_key_names() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.json");
    fs::write(&data, r#"{"123": "numeric key"}"#).unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ 123 }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("numeric key\n");
}

/// エッジケース: 浮動小数点数
#[test]
fn test_floating_point_numbers() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "pi: 3.14159\nlarge: 1.23e10\nsmall: 1.23e-10").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ pi }}, {{ large }}, {{ small }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout(predicate::str::contains("3.14159"));
}

/// エッジケース: 非常に大きな数値
#[test]
fn test_large_numbers() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "big: 999999999999999999").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ big }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout(predicate::str::contains("999999999999999999"));
}

/// エッジケース: バイナリデータ（Base64エンコード）
#[test]
fn test_base64_data() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "binary: \"SGVsbG8gV29ybGQh\"").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ binary }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("SGVsbG8gV29ybGQh\n");
}

/// エッジケース: 複数の改行コードの混在
#[test]
fn test_mixed_line_endings() {
    let dir = tempdir().unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "Line1\nLine2\rLine3\r\nLine4").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .assert()
        .success();
}

/// エッジケース: UTF-8 BOM付きファイル
#[test]
fn test_utf8_bom() {
    let dir = tempdir().unwrap();

    let template = dir.path().join("template.txt");
    let mut content = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
    content.extend_from_slice(b"Test");
    fs::write(&template, content).unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .assert()
        .success();
}

/// エッジケース: 絵文字シーケンス
#[test]
fn test_emoji_sequences() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "emoji: \"👨‍👩‍👧‍👦🏳️‍🌈\"").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ emoji }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("👨‍👩‍👧‍👦🏳️‍🌈\n");
}
