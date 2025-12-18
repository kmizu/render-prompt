# render-prompt

シンプルで強力なテンプレートレンダリングツール。YAML/JSONデータから変数置換とファイルインクルードを行い、プレーンテキストを生成します。

## 特徴

- **最小限の構文**: 変数置換 `{{ var }}` とインクルード `{{> file }}` のみ
- **データマージ**: 複数のYAML/JSONファイルを自動マージ
- **安全性**: パストラバーサル防止、循環インクルード検出
- **高速**: Rust製、シングルバイナリ
- **クロスプラットフォーム**: macOS、Linux、Windows対応

## インストール

### ソースからビルド

```bash
cargo build --release
```

バイナリは `target/release/render-prompt` に生成されます。

## 使い方

### 基本的な使い方

```bash
render-prompt --template template.txt --data data.yaml
```

### 実例

**data.yaml:**
```yaml
name: Alice
age: 30
segments:
  engineer:
    title: "バックエンドエンジニア"
    pain: "仕様変更が多い"
```

**header.txt:**
```
=== {{ name }}さんのプロフィール ===
年齢: {{ age }}
```

**template.txt:**
```
{{> header.txt }}

# セグメント情報

## {{ segments.engineer.title }}
課題: {{ segments.engineer.pain }}

変数を参照するには \{{ variable }} を使います。
```

**実行:**
```bash
render-prompt --template template.txt --data data.yaml
```

**出力:**
```
=== Aliceさんのプロフィール ===
年齢: 30

# セグメント情報

## バックエンドエンジニア
課題: 仕様変更が多い

変数を参照するには {{ variable }} を使います。
```

## コマンドラインオプション

### 必須オプション

| オプション | 短縮形 | 説明 |
|-----------|-------|------|
| `--template <PATH>` | `-t` | テンプレートファイルのパス |

### データオプション

| オプション | 短縮形 | 説明 |
|-----------|-------|------|
| `--data <PATH>` | `-d` | データファイル（YAML/JSON）。複数指定可能 |

複数のデータファイルを指定すると、Deep mergeで結合されます（後勝ち）：

```bash
render-prompt -t template.txt -d base.yaml -d prod.yaml
```

### 出力オプション

| オプション | 短縮形 | 説明 |
|-----------|-------|------|
| `--out <PATH>` | `-o` | 出力ファイルのパス。未指定時は標準出力 |

```bash
render-prompt -t template.txt -d data.yaml -o output.txt
```

### インクルード設定

| オプション | 説明 | デフォルト |
|-----------|------|-----------|
| `--root <DIR>` | インクルードファイルの探索ルートディレクトリ | テンプレートのディレクトリ |
| `--max-include-depth <N>` | インクルードの最大深さ | 20 |

```bash
render-prompt -t template.txt -d data.yaml --root ./templates --max-include-depth 10
```

### エラー処理オプション

| オプション | 説明 |
|-----------|------|
| `--strict` | 未定義変数をエラーとして扱う |
| `--warn-undefined` | 未定義変数を警告表示（stderrに出力） |

```bash
# 未定義変数でエラー終了
render-prompt -t template.txt -d data.yaml --strict

# 未定義変数を警告表示
render-prompt -t template.txt -d data.yaml --warn-undefined
```

## テンプレート構文

### 変数置換

#### 基本構文

```
Hello, {{ name }}!
```

- 変数名の前後の空白は無視されます: `{{name}}` と `{{ name }}` は同じ
- 未定義変数はデフォルトで空文字に置換されます

#### ネストしたオブジェクト

ドット記法でネストした値にアクセスできます：

```
{{ user.profile.name }}
{{ company.address.city }}
```

#### 配列アクセス

0始まりのインデックスで配列要素にアクセスできます：

```
{{ items.0 }}
{{ matrix.1.2 }}
```

#### データ型の扱い

| データ型 | 出力例 |
|---------|--------|
| 文字列 | そのまま出力 |
| 数値 | `30` |
| 真偽値 | `true` または `false` |
| null | 空文字 |
| オブジェクト/配列 | JSON文字列 |

#### エスケープ

`{{` をそのまま出力したい場合は、バックスラッシュでエスケープします：

```
\{{ これはそのまま出力されます }}
```

出力: `{{ これはそのまま出力されます }}`

### インクルードディレクティブ

#### 基本構文

```
{{> path/to/file.txt }}
```

- 相対パスで指定します
- インクルードされたファイル内でも変数置換とインクルードが再帰的に処理されます

#### インクルードの例

**partials/header.txt:**
```
# {{ title }}
---
```

**template.txt:**
```
{{> partials/header.txt }}

本文: {{ content }}
```

#### ネストしたインクルード

インクルードは再帰的に処理されます：

```
main.txt
  └─> header.txt
       └─> logo.txt
```

**制限事項:**
- 循環インクルードは自動検出されエラーになります
- 深さ制限を超えるとエラーになります（デフォルト: 20）
- `--root` で指定したディレクトリ外へのアクセスは禁止されます

## データファイル形式

### YAML

```yaml
name: Alice
age: 30
hobbies:
  - reading
  - coding
profile:
  bio: "Software Engineer"
  location: "Tokyo"
```

### JSON

```json
{
  "name": "Alice",
  "age": 30,
  "hobbies": ["reading", "coding"],
  "profile": {
    "bio": "Software Engineer",
    "location": "Tokyo"
  }
}
```

### 複数ファイルのマージ

**base.yaml:**
```yaml
app:
  name: "MyApp"
  version: "1.0.0"
```

**prod.yaml:**
```yaml
app:
  version: "1.0.1"
  env: "production"
```

```bash
render-prompt -t template.txt -d base.yaml -d prod.yaml
```

**結果:**
```yaml
app:
  name: "MyApp"        # base.yamlから
  version: "1.0.1"     # prod.yamlで上書き
  env: "production"    # prod.yamlで追加
```

**マージルール:**
- オブジェクトは再帰的にマージされます
- 配列は後のファイルで完全に上書きされます
- プリミティブ値は後のファイルが優先されます

## 終了コード

| コード | 説明 |
|-------|------|
| 0 | 成功 |
| 2 | コマンドライン引数エラー |
| 3 | テンプレートファイル読み込みエラー |
| 4 | データファイル読み込み/パースエラー |
| 5 | インクルードファイルエラー |
| 6 | 変数解決エラー（strict モード） |
| 7 | 循環インクルード/深さ制限超過 |

## エラーメッセージ

エラーメッセージは機械可読な形式で標準エラー出力に出力されます：

```
ERROR code=UNDEFINED_VAR var="user.email" template="template.txt" line=12 col=5
Undefined variable 'user.email' at template.txt:12:5
```

## 実用例

### プロンプトテンプレート管理

AIプロンプトを管理する場合：

```bash
render-prompt \
  --template prompts/base.txt \
  --data config/segments.yaml \
  --data config/examples.yaml \
  --out prompts/generated/final.txt
```

### 環境別設定生成

```bash
# 開発環境
render-prompt -t config.template -d base.yaml -d dev.yaml -o config.dev

# 本番環境
render-prompt -t config.template -d base.yaml -d prod.yaml -o config.prod
```

### ドキュメント生成

```bash
render-prompt \
  --template docs/template.md \
  --data project-info.yaml \
  --out README.md
```

## 制限事項

render-promptは意図的にシンプルに保たれています。以下の機能は**サポートされていません**：

- ❌ 条件分岐（if/else）
- ❌ ループ（for/each）
- ❌ 関数やフィルター
- ❌ 数式評価
- ❌ カスタム関数
- ❌ ネットワークアクセス
- ❌ コード実行

複雑なロジックが必要な場合は、データファイル側で事前に処理してください。

## 開発

### テスト実行

```bash
# 全テスト実行
cargo test

# 特定のモジュールのテスト
cargo test data::
cargo test template::

# リリースビルドでテスト
cargo test --release
```

### ビルド

```bash
# デバッグビルド
cargo build

# リリースビルド（最適化あり）
cargo build --release
```

### プロジェクト構成

```
render-prompt/
├── src/
│   ├── main.rs          # エントリーポイント
│   ├── cli.rs           # CLI定義
│   ├── error.rs         # エラー型
│   ├── data/            # データローダー
│   │   ├── mod.rs
│   │   ├── loader.rs    # YAML/JSON読み込み
│   │   └── merger.rs    # Deep merge
│   └── template/        # テンプレートエンジン
│       ├── mod.rs
│       ├── engine.rs    # メインエンジン
│       ├── variable.rs  # 変数置換
│       └── include.rs   # インクルード処理
├── tests/               # 統合テスト
└── test_data/           # テストデータ
```

## ライセンス

MIT

## 貢献

Issue、Pull Requestは大歓迎です！

## 関連プロジェクト

- [Mustache](https://mustache.github.io/) - ロジックレステンプレート
- [Handlebars](https://handlebarsjs.com/) - 拡張可能なテンプレート
- [Jinja2](https://jinja.palletsprojects.com/) - Pythonテンプレートエンジン

render-promptはこれらのツールからインスピレーションを得ていますが、より最小限の機能セットに絞っています。
