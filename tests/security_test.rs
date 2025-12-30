use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

/// セキュリティ: パストラバーサル防止 - 親ディレクトリへのアクセス
#[test]
fn test_path_traversal_parent_directory() {
    let dir = tempdir().unwrap();
    let subdir = dir.path().join("templates");
    fs::create_dir(&subdir).unwrap();

    // rootの外にファイルを作成
    let outside = dir.path().join("secret.txt");
    fs::write(&outside, "SECRET DATA").unwrap();

    // テンプレートはsubdir内
    let template = subdir.join("template.txt");
    fs::write(&template, "{{> ../secret.txt }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("--root")
        .arg(&subdir)
        .assert()
        .failure()
        .code(5); // EXIT_INCLUDE_ERROR
}

/// セキュリティ: パストラバーサル防止 - 絶対パス
#[test]
fn test_path_traversal_absolute_path() {
    let dir = tempdir().unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{> /etc/passwd }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .assert()
        .failure();
}

/// セキュリティ: パストラバーサル防止 - 複数の../
#[test]
fn test_path_traversal_multiple_parents() {
    let dir = tempdir().unwrap();
    let level1 = dir.path().join("level1");
    let level2 = level1.join("level2");
    let level3 = level2.join("level3");
    fs::create_dir_all(&level3).unwrap();

    let outside = dir.path().join("secret.txt");
    fs::write(&outside, "SECRET").unwrap();

    let template = level3.join("template.txt");
    fs::write(&template, "{{> ../../../secret.txt }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("--root")
        .arg(&level3)
        .assert()
        .failure()
        .code(5);
}

/// セキュリティ: パストラバーサル防止 - URL風のパス
#[test]
fn test_path_traversal_url_like() {
    let dir = tempdir().unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{> file://etc/passwd }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .assert()
        .failure();
}

/// セキュリティ: パストラバーサル防止 - シンボリックリンク
#[cfg(unix)]
#[test]
fn test_path_traversal_symlink() {
    use std::os::unix::fs::symlink;

    let dir = tempdir().unwrap();
    let templates = dir.path().join("templates");
    fs::create_dir(&templates).unwrap();

    let outside = dir.path().join("secret.txt");
    fs::write(&outside, "SECRET").unwrap();

    // シンボリックリンクを作成
    let link = templates.join("link.txt");
    symlink(&outside, &link).unwrap();

    let template = templates.join("template.txt");
    fs::write(&template, "{{> link.txt }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("--root")
        .arg(&templates)
        .assert()
        .failure()
        .code(5);
}

/// セキュリティ: DoS対策 - 循環includeの検出
#[test]
fn test_dos_circular_include_immediate() {
    let dir = tempdir().unwrap();

    let file = dir.path().join("file.txt");
    fs::write(&file, "{{> file.txt }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&file)
        .assert()
        .failure()
        .code(7); // EXIT_CIRCULAR_OR_DEPTH_ERROR
}

/// セキュリティ: DoS対策 - 深さ制限
#[test]
fn test_dos_depth_limit() {
    let dir = tempdir().unwrap();

    // 深い階層のincludeを作成
    for i in 0..100 {
        let file = dir.path().join(format!("{}.txt", i));
        if i < 99 {
            fs::write(&file, format!("{{{{> {}.txt }}}}", i + 1)).unwrap();
        } else {
            fs::write(&file, "end").unwrap();
        }
    }

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(dir.path().join("0.txt"))
        .arg("--max-include-depth")
        .arg("10")
        .assert()
        .failure()
        .code(7);
}

/// セキュリティ: 巨大ファイルの扱い
#[test]
fn test_large_file_handling() {
    let dir = tempdir().unwrap();

    // 1MBのテンプレートファイル
    let large_content = "x".repeat(1024 * 1024);
    let template = dir.path().join("template.txt");
    fs::write(&template, large_content.clone()).unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .assert()
        .success()
        .stdout(format!("{}\n", large_content));
}

/// セキュリティ: 特殊文字を含むファイル名
#[test]
fn test_special_characters_in_filename() {
    let dir = tempdir().unwrap();

    let included = dir.path().join("file_with_special_!@#.txt");
    fs::write(&included, "content").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{> file_with_special_!@#.txt }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .assert()
        .success()
        .stdout("content\n");
}

/// セキュリティ: NULL文字の扱い
#[test]
fn test_null_character_handling() {
    let dir = tempdir().unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "Before\0After").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .assert()
        .success();
}

/// セキュリティ: コマンドインジェクション防止 (ファイル名)
#[test]
fn test_command_injection_filename() {
    let dir = tempdir().unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{> file; rm -rf / }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .assert()
        .failure(); // ファイルが見つからないエラー
}

/// セキュリティ: スクリプトインジェクション防止
#[test]
fn test_script_injection_in_data() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, r#"script: "<script>alert('XSS')</script>""#).unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ script }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("<script>alert('XSS')</script>\n"); // エスケープせずそのまま出力
}

/// セキュリティ: SQLインジェクション風の文字列
#[test]
fn test_sql_injection_like_string() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, r#"query: "'; DROP TABLE users; --""#).unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ query }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("'; DROP TABLE users; --\n");
}

/// セキュリティ: 環境変数の参照防止
#[test]
fn test_no_environment_variable_access() {
    let dir = tempdir().unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ $HOME }}{{ env.HOME }}{{ ENV.HOME }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .assert()
        .success()
        .stdout("\n"); // 全て空文字
}

/// セキュリティ: ファイルパスの正規化
#[test]
fn test_path_normalization() {
    let dir = tempdir().unwrap();

    let included = dir.path().join("file.txt");
    fs::write(&included, "content").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{> ./file.txt }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .assert()
        .success()
        .stdout("content\n");
}

/// セキュリティ: 複雑なパス操作
#[test]
fn test_complex_path_manipulation() {
    let dir = tempdir().unwrap();
    let subdir = dir.path().join("sub");
    fs::create_dir(&subdir).unwrap();

    let file = subdir.join("file.txt");
    fs::write(&file, "content").unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{> ./sub/../sub/./file.txt }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .assert()
        .success()
        .stdout("content\n");
}

/// セキュリティ: ディレクトリトラバーサルの複雑なパターン
#[test]
fn test_complex_directory_traversal() {
    let dir = tempdir().unwrap();
    let allowed = dir.path().join("allowed");
    fs::create_dir(&allowed).unwrap();

    let outside = dir.path().join("secret.txt");
    fs::write(&outside, "SECRET").unwrap();

    let template = allowed.join("template.txt");
    fs::write(&template, "{{> ./../secret.txt }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("--root")
        .arg(&allowed)
        .assert()
        .failure()
        .code(5);
}

/// セキュリティ: Zip Slip風の攻撃
#[test]
fn test_zip_slip_like_attack() {
    let dir = tempdir().unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{> ../../../../../../../../etc/passwd }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .assert()
        .failure();
}

/// セキュリティ: 制御文字を含むデータ
#[test]
fn test_control_characters() {
    let dir = tempdir().unwrap();

    // YAMLは生の制御文字を許可しないため、JSONを使用
    let data = dir.path().join("data.json");
    fs::write(&data, "{\"text\": \"line1\\u001b[31mred\\u001b[0m\"}").unwrap();

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
        .stdout("line1\u{001b}[31mred\u{001b}[0m\n");
}

/// セキュリティ: バックスラッシュの扱い
#[test]
fn test_backslash_handling() {
    let dir = tempdir().unwrap();

    let data = dir.path().join("data.yaml");
    fs::write(&data, r#"path: "C:\\Users\\Test""#).unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ path }}").unwrap();

    Command::cargo_bin("rp")
        .unwrap()
        .arg("-t")
        .arg(&template)
        .arg("-d")
        .arg(&data)
        .assert()
        .success()
        .stdout("C:\\Users\\Test\n");
}

/// セキュリティ: 再帰的なオブジェクト参照（JSONの場合）
#[test]
fn test_no_recursive_object_reference() {
    let dir = tempdir().unwrap();

    // 通常のJSONは再帰参照を持てないが、念のためテスト
    let data = dir.path().join("data.json");
    fs::write(&data, r#"{"obj": {"self": "value"}}"#).unwrap();

    let template = dir.path().join("template.txt");
    fs::write(&template, "{{ obj.self }}").unwrap();

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
