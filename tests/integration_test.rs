use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

/// CLI統合テスト: 基本的な実行
#[test]
fn test_basic_execution() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "name: Alice\nage: 30").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "Hello, {{ name }}! Age: {{ age }}").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("--template")
        .arg(&template)
        .arg("--data")
        .arg(&data)
        .assert()
        .success()
        .stdout("Hello, Alice! Age: 30\n");
}

/// CLI統合テスト: 複数データファイルのマージ
#[test]
fn test_multiple_data_files_merge() {
    let dir = tempdir().unwrap();

    let data1 = dir.path().join("data1.yaml");
    fs::write(&data1, "name: Alice\nage: 30\nhobbies:\n  - reading").unwrap();

    let data2 = dir.path().join("data2.yaml");
    fs::write(&data2, "age: 31\ncity: Tokyo\nhobbies:\n  - coding").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(
        &template,
        "{{ name }}, {{ age }}, {{ city }}, {{ hobbies.0 }}",
    )
    .unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data1)
        .arg("-d")
        .arg(&data2)
        .assert()
        .success()
        .stdout("Alice, 31, Tokyo, coding\n");
}

/// CLI統合テスト: 出力ファイル
#[test]
fn test_output_file() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "message: Hello World").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ message }}").unwrap();

    let output = dir.path().join("output.txt");

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .arg("-o")
        .arg(&output)
        .assert()
        .success();

    let content = fs::read_to_string(&output).unwrap();
    assert_eq!(content, "Hello World");
}

/// CLI統合テスト: strictモードで未定義変数エラー
#[test]
fn test_strict_mode_undefined_variable() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "name: Alice").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ name }} {{ undefined }}").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .arg("--strict")
        .assert()
        .failure()
        .code(6) // EXIT_VARIABLE_ERROR
        .stderr(predicate::str::contains("undefined"));
}

/// CLI統合テスト: 非strictモードで未定義変数は空文字
#[test]
fn test_non_strict_mode_undefined_variable() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "name: Alice").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "Hello {{ name }}{{ undefined }}!").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("Hello Alice!\n");
}

/// CLI統合テスト: includeの基本動作
#[test]
fn test_include_basic() {
    let dir = tempdir().unwrap();

    let included = dir.path().join("included.txt");
    fs::write(&included, "Included content: {{ value }}").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "Start\n{{> included.txt }}\nEnd").unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "value: 42").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("Start\nIncluded content: 42\nEnd\n");
}

/// CLI統合テスト: ネストしたinclude
#[test]
fn test_nested_includes() {
    let dir = tempdir().unwrap();

    let level3 = dir.path().join("level3.txt");
    fs::write(&level3, "Level 3: {{ var3 }}").unwrap();

    let level2 = dir.path().join("level2.txt");
    fs::write(&level2, "Level 2: {{ var2 }}\n{{> level3.txt }}").unwrap();

    let level1 = dir.path().join("level1.txt");
    fs::write(&level1, "Level 1: {{ var1 }}\n{{> level2.txt }}").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "Start\n{{> level1.txt }}\nEnd").unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "var1: A\nvar2: B\nvar3: C").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("Start\nLevel 1: A\nLevel 2: B\nLevel 3: C\nEnd\n");
}

/// CLI統合テスト: 循環includeでエラー
#[test]
fn test_circular_include_error() {
    let dir = tempdir().unwrap();

    let file_a = dir.path().join("a.txt");
    let file_b = dir.path().join("b.txt");

    fs::write(&file_a, "A {{> b.txt }}").unwrap();
    fs::write(&file_b, "B {{> a.txt }}").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&file_a)
        .assert()
        .failure()
        .code(7) // EXIT_CIRCULAR_OR_DEPTH_ERROR
        .stderr(predicate::str::contains("Circular include"));
}

/// CLI統合テスト: 深さ制限超過
#[test]
fn test_depth_limit_exceeded() {
    let dir = tempdir().unwrap();

    // 深いネストのincludeチェーンを作成
    for i in 0..15 {
        let file = dir.path().join(format!("{}.txt", i));
        if i < 14 {
            fs::write(&file, format!("Level {}\n{{{{> {}.txt }}}}", i, i + 1)).unwrap();
        } else {
            fs::write(&file, format!("Level {}", i)).unwrap();
        }
    }

    let template = dir.path().join("0.txt");

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("--max-include-depth")
        .arg("5")
        .assert()
        .failure()
        .code(7)
        .stderr(predicate::str::contains("depth"));
}

/// CLI統合テスト: 存在しないテンプレートファイル
#[test]
fn test_nonexistent_template() {
    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg("/nonexistent/template.txt")
        .assert()
        .failure()
        .code(3); // EXIT_TEMPLATE_ERROR
}

/// CLI統合テスト: 存在しないデータファイル
#[test]
fn test_nonexistent_data_file() {
    let dir = tempdir().unwrap();
    let template = dir.path().join("template.txt");
    fs::write(&template, "Hello").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg("/nonexistent/data.yaml")
        .assert()
        .failure()
        .code(4); // EXIT_DATA_ERROR
}

/// CLI統合テスト: 不正なYAML
#[test]
fn test_invalid_yaml() {
    let dir = tempdir().unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "Test").unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "invalid: yaml: syntax: error:").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .failure()
        .code(4);
}

/// CLI統合テスト: 不正なJSON
#[test]
fn test_invalid_json() {
    let dir = tempdir().unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "Test").unwrap();

    let data = dir.path().join("data.json");
    fs::write(&data, "{invalid json}").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .failure()
        .code(4);
}

/// CLI統合テスト: エスケープの動作
#[test]
fn test_escape_syntax() {
    let dir = tempdir().unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, r"{{ var }} and \{{ not_a_var }}").unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "var: Hello").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("Hello and {{ not_a_var }}\n");
}

/// CLI統合テスト: 複雑なネストしたデータ構造
#[test]
fn test_complex_nested_data() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(
        &data,
        r#"
company:
  name: "TechCorp"
  departments:
    - name: "Engineering"
      head: "Alice"
      members: 50
    - name: "Sales"
      head: "Bob"
      members: 30
  location:
    city: "Tokyo"
    country: "Japan"
"#,
    )
    .unwrap();

    let template = dir.path().join("template.txt");
    fs::write(
        &template,
        r#"Company: {{ company.name }}
Dept: {{ company.departments.0.name }} (Head: {{ company.departments.0.head }}, {{ company.departments.0.members }} members)
Dept: {{ company.departments.1.name }} (Head: {{ company.departments.1.head }}, {{ company.departments.1.members }} members)
Location: {{ company.location.city }}, {{ company.location.country }}"#,
    )
    .unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout(
            "Company: TechCorp
Dept: Engineering (Head: Alice, 50 members)
Dept: Sales (Head: Bob, 30 members)
Location: Tokyo, Japan
",
        );
}

/// CLI統合テスト: JSON形式のデータ
#[test]
fn test_json_data_format() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.json");
    fs::write(
        &data,
        r#"{"user": {"name": "Charlie", "age": 25}, "active": true}"#,
    )
    .unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ user.name }}, {{ user.age }}, {{ active }}").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("Charlie, 25, true\n");
}

/// CLI統合テスト: YAMLとJSONの混在
#[test]
fn test_mixed_yaml_and_json() {
    let dir = tempdir().unwrap();

    let yaml_data = dir.path().join("data.yaml");
    fs::write(&yaml_data, "from_yaml: YAML").unwrap();

    let json_data = dir.path().join("data.json");
    fs::write(&json_data, r#"{"from_json": "JSON"}"#).unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ from_yaml }} and {{ from_json }}").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&yaml_data)
        .arg("-d")
        .arg(&json_data)
        .assert()
        .success()
        .stdout("YAML and JSON\n");
}

/// CLI統合テスト: 空のデータファイル
#[test]
fn test_empty_data_file() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "No data: {{ missing }}").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("No data: \n");
}

/// CLI統合テスト: データファイルなし
#[test]
fn test_no_data_file() {
    let dir = tempdir().unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "No variables").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .assert()
        .success()
        .stdout("No variables\n");
}

/// CLI統合テスト: 空のテンプレート
#[test]
fn test_empty_template() {
    let dir = tempdir().unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .assert()
        .success()
        .stdout("\n");
}

/// CLI統合テスト: 大量の変数
#[test]
fn test_many_variables() {
    let dir = tempdir().unwrap();

    let mut data_content = String::new();
    for i in 0..100 {
        data_content.push_str(&format!("var{}: value{}\n", i, i));
    }
    let data = dir.path().join("data.yaml");
    fs::write(&data, data_content).unwrap();

    let mut template_content = String::new();
    for i in 0..100 {
        template_content.push_str(&format!("{{{{ var{} }}}} ", i));
    }
    let template = dir.path().join("template.txt");
    fs::write(&template, template_content).unwrap();

    let mut expected = String::new();
    for i in 0..100 {
        expected.push_str(&format!("value{} ", i));
    }
    expected.push('\n');

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout(expected);
}

/// CLI統合テスト: rootオプションでinclude探索
#[test]
fn test_root_option() {
    let dir = tempdir().unwrap();
    let includes_dir = dir.path().join("includes");
    fs::create_dir(&includes_dir).unwrap();

    let included = includes_dir.join("part.txt");
    fs::write(&included, "Included: {{ value }}").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{> includes/part.txt }}").unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "value: 123").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .arg("--root")
        .arg(dir.path())
        .assert()
        .success()
        .stdout("Included: 123\n");
}

/// CLI統合テスト: 特殊文字を含むデータ
#[test]
fn test_special_characters() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(
        &data,
        r#"special: "Hello \"World\" with 'quotes' and\nnewlines""#,
    )
    .unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ special }}").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success();
}

/// CLI統合テスト: Unicode文字
#[test]
fn test_unicode_characters() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "message: こんにちは世界\nemoji: 🎉").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ message }} {{ emoji }}").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("こんにちは世界 🎉\n");
}

/// CLI統合テスト: オブジェクトのJSON出力
#[test]
fn test_object_as_json() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "obj:\n  key1: value1\n  key2: value2").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "Object: {{ obj }}").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout(predicate::str::contains("key1"))
        .stdout(predicate::str::contains("value1"));
}

/// CLI統合テスト: 配列のJSON出力
#[test]
fn test_array_as_json() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, "items:\n  - one\n  - two\n  - three").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "Array: {{ items }}").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout(predicate::str::contains("["))
        .stdout(predicate::str::contains("one"));
}

/// CLI統合テスト: バージョン表示
#[test]
fn test_version_flag() {
    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("render-prompt"));
}

/// CLI統合テスト: ヘルプ表示
#[test]
fn test_help_flag() {
    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("template"))
        .stdout(predicate::str::contains("data"));
}

/// CLI統合テスト: 引数不足
#[test]
fn test_missing_required_args() {
    Command::cargo_bin("render-prompt")
        .unwrap()
        .assert()
        .failure()
        .code(2); // EXIT_USAGE_ERROR
}

/// CLI統合テスト: max-include-depthの検証
#[test]
fn test_invalid_max_depth_zero() {
    let dir = tempdir().unwrap();
    let template = dir.path().join("template.txt");
    fs::write(&template, "test").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("--max-include-depth")
        .arg("0")
        .assert()
        .failure()
        .code(2);
}

/// CLI統合テスト: max-include-depthが大きすぎる
#[test]
fn test_invalid_max_depth_too_large() {
    let dir = tempdir().unwrap();
    let template = dir.path().join("template.txt");
    fs::write(&template, "test").unwrap();

    Command::cargo_bin("render-prompt")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("--max-include-depth")
        .arg("1001")
        .assert()
        .failure()
        .code(2);
}
