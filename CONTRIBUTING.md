# 開発ガイド

render-promptへの貢献に興味を持っていただき、ありがとうございます！

## 開発環境のセットアップ

### 必要なもの

- Rust 1.70 以上
- Cargo

### セットアップ手順

```bash
# リポジトリのクローン
git clone https://github.com/yourusername/render-prompt.git
cd render-prompt

# 依存関係のインストールとビルド
cargo build

# テスト実行
cargo test

# リリースビルド
cargo build --release
```

## プロジェクト構成

```
render-prompt/
├── src/
│   ├── main.rs              # エントリーポイント
│   ├── cli.rs               # CLI定義（clap）
│   ├── error.rs             # エラー型・終了コード
│   ├── data/                # データ処理モジュール
│   │   ├── mod.rs
│   │   ├── loader.rs        # YAML/JSON読み込み
│   │   └── merger.rs        # Deep merge実装
│   └── template/            # テンプレートエンジン
│       ├── mod.rs
│       ├── engine.rs        # メインエンジン
│       ├── variable.rs      # 変数置換処理
│       └── include.rs       # Include処理
├── tests/                   # 統合テスト（今後追加予定）
├── test_data/               # 手動テスト用データ
├── README.md                # プロジェクト概要
├── EXAMPLES.md              # 使用例集
└── CONTRIBUTING.md          # このファイル
```

## アーキテクチャ

### 処理フロー

render-promptは以下の順序でテンプレートを処理します：

```
1. CLI引数のパース（cli.rs）
   ↓
2. データファイルの読み込み・マージ（data/loader.rs, data/merger.rs）
   ↓
3. テンプレートファイルの読み込み（template/engine.rs）
   ↓
4. Includeディレクティブの解決（template/include.rs）
   ↓
5. 変数置換の実行（template/variable.rs）
   ↓
6. 結果の出力（main.rs）
```

### 主要コンポーネント

#### 1. データローダー（data/）

**loader.rs**: YAML/JSONファイルを読み込み、`serde_json::Value`に変換

```rust
pub fn load_file(path: &Path) -> Result<Value, RenderError>
pub fn load_multiple(paths: &[P]) -> Result<Value, RenderError>
```

**merger.rs**: 複数のデータを再帰的にマージ

```rust
pub fn merge(base: &mut Value, overlay: &Value)
pub fn merge_multiple(values: Vec<Value>) -> Value
```

マージルール：
- オブジェクト: キーごとに再帰的マージ（後勝ち）
- 配列: 後のファイルで完全上書き
- プリミティブ: 後勝ち

#### 2. テンプレートエンジン（template/）

**engine.rs**: 全体のオーケストレーション

```rust
pub fn render(&self, template_path: &Path, data: &Value) -> Result<String, RenderError>
```

処理順序（仕様書で規定）：
1. テンプレート読み込み
2. Include解決（再帰）
3. 変数置換（一括）
4. エスケープ処理

**include.rs**: `{{> path }}` の処理

```rust
pub fn resolve(
    &self,
    content: &str,
    current_file: &Path,
    visited: &mut HashSet<PathBuf>,
    depth: usize,
) -> Result<String, RenderError>
```

重要な機能：
- 循環検出（`visited` セット）
- 深さ制限（デフォルト20）
- パストラバーサル防止（`canonicalize`）

**variable.rs**: `{{ var }}` の処理

```rust
pub fn substitute(&self, content: &str, data: &Value) -> Result<String, RenderError>
```

機能：
- ドットパス解決（`user.name`）
- 配列アクセス（`items.0`）
- エスケープ（`\{{` → `{{`）
- Strict/非strictモード

#### 3. エラー処理（error.rs）

```rust
pub enum RenderError {
    DataFileRead { path: String, source: std::io::Error },
    DataFileParse { path: String, source: anyhow::Error },
    UndefinedVariable { name: String, location: Location },
    CircularInclude { path: String },
    IncludeDepthExceeded { max_depth: usize },
    PathTraversal { path: String },
    // ...
}
```

各エラーは対応する終了コードを持ちます（2-7）。

## テスト

### ユニットテスト

各モジュール内に`#[cfg(test)]`でテストを記述：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_merge() {
        let mut base = json!({ "a": 1 });
        let overlay = json!({ "b": 2 });
        DataMerger::merge(&mut base, &overlay);
        assert_eq!(base, json!({ "a": 1, "b": 2 }));
    }
}
```

### テストの実行

```bash
# 全テスト実行
cargo test

# 特定モジュールのテスト
cargo test data::merger

# テスト名で絞り込み
cargo test merge

# 詳細出力
cargo test -- --nocapture

# リリースビルドでテスト
cargo test --release
```

### テストのベストプラクティス

1. **境界値テスト**: 空文字列、null、空配列など
2. **エラーケース**: 各エラー型を網羅
3. **エッジケース**: 循環参照、深いネスト、特殊文字
4. **一時ファイル**: `tempfile`クレートを使用

```rust
use tempfile::NamedTempFile;

#[test]
fn test_load_yaml() {
    let mut file = NamedTempFile::with_suffix(".yaml").unwrap();
    writeln!(file, "key: value").unwrap();

    let result = DataLoader::load_file(file.path()).unwrap();
    assert_eq!(result, json!({"key": "value"}));
}
```

## コーディング規約

### Rustスタイル

- `rustfmt`を使用（デフォルト設定）
- `clippy`の警告に対処

```bash
# フォーマット
cargo fmt

# Lint
cargo clippy

# 全ての警告を表示
cargo clippy -- -W clippy::all
```

### 命名規則

- **関数**: `snake_case`
- **構造体/列挙型**: `PascalCase`
- **定数**: `SCREAMING_SNAKE_CASE`
- **モジュール**: `snake_case`

### エラー処理

- `Result<T, RenderError>`を使用
- `?`演算子で伝播
- `thiserror`でエラー定義

```rust
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("Failed to read file '{path}': {source}")]
    DataFileRead {
        path: String,
        source: std::io::Error,
    },
}
```

### ドキュメント

パブリックAPIには必ずドキュメントコメントを付ける：

```rust
/// Load a single data file (YAML or JSON)
///
/// # Arguments
///
/// * `path` - Path to the data file
///
/// # Returns
///
/// * `Ok(Value)` - Parsed JSON value
/// * `Err(RenderError)` - If file cannot be read or parsed
pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Value, RenderError> {
    // ...
}
```

## 貢献の流れ

### 1. Issueの作成

バグ報告や機能提案は、まずIssueを作成してください。

**バグ報告テンプレート:**
```
## 問題の説明
バグの内容を簡潔に説明

## 再現手順
1. `rp -t ...`を実行
2. エラーが発生

## 期待される動作
正常に動作するはず

## 実際の動作
エラーメッセージが表示される

## 環境
- OS: macOS 14.0
- Rust: 1.75.0
- rp: 0.1.0
```

### 2. ブランチの作成

```bash
# 最新のmainを取得
git checkout main
git pull origin main

# 機能ブランチを作成
git checkout -b feature/your-feature-name

# または、バグ修正の場合
git checkout -b fix/bug-description
```

### 3. 実装

1. コードを書く
2. テストを追加
3. ドキュメントを更新
4. `cargo fmt`でフォーマット
5. `cargo clippy`で警告をチェック
6. `cargo test`で全テスト実行

### 4. コミット

コミットメッセージは明確に：

```bash
git add .
git commit -m "feat: 新機能の追加"
```

コミットメッセージのプレフィックス：
- `feat:` 新機能
- `fix:` バグ修正
- `docs:` ドキュメント
- `test:` テスト追加
- `refactor:` リファクタリング
- `perf:` パフォーマンス改善
- `chore:` その他の変更

### 5. Pull Request

```bash
git push origin feature/your-feature-name
```

GitHubでPull Requestを作成。以下を含めてください：

- 変更内容の説明
- 関連するIssue番号
- テスト結果
- スクリーンショット（UI変更の場合）

## 機能追加のガイドライン

### 原則

rpは**最小限の機能セット**を維持します：

✅ **受け入れられる機能:**
- パフォーマンス改善
- エラーメッセージの改善
- セキュリティ向上
- 既存機能のバグ修正

❌ **受け入れられない機能:**
- 条件分岐（if/else）
- ループ（for/each）
- 関数やフィルター
- カスタムスクリプト実行

複雑なロジックはデータ側で処理することを推奨しています。

### 提案前のチェックリスト

- [ ] 既存の機能で実現できないか検討した
- [ ] データ側で解決できないか検討した
- [ ] 他のツール（Mustache、Jinja2など）でも実現困難か確認した
- [ ] rpの設計思想に沿っているか確認した

## デバッグ

### ログ出力

開発時は`RUST_LOG`環境変数を設定：

```bash
RUST_LOG=debug cargo run -- -t template.txt -d data.yaml
```

### デバッガーの使用

VS Codeの場合、`.vscode/launch.json`を作成：

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug rp",
      "cargo": {
        "args": ["build", "--bin=rp"]
      },
      "args": [
        "--template", "test_data/template.txt",
        "--data", "test_data/data.yaml"
      ],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

## パフォーマンス

### ベンチマーク

```bash
# リリースビルドで実行
cargo build --release

# 大きなファイルでテスト
time ./target/release/rp \
  -t large_template.txt \
  -d large_data.yaml
```

### 最適化のヒント

1. **正規表現のコンパイル**: `lazy_static`で事前コンパイル
2. **String割り当て**: `with_capacity`で事前確保
3. **不要なクローン**: 参照を使う
4. **ファイルI/O**: `read_to_string`を一度だけ

## リリースプロセス

1. バージョン番号を更新（`Cargo.toml`）
2. CHANGELOGを更新
3. 全テストが通ることを確認
4. タグを作成

```bash
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0
```

## 質問やサポート

- GitHub Issueで質問してください
- Discussionsで議論を開始できます

## ライセンス

貢献したコードはMITライセンスの下で公開されます。

---

貢献していただき、ありがとうございます！🎉
