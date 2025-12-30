# rp 使用例集

このドキュメントでは、rpの実践的な使用例を紹介します。

## 目次

1. [基本的な使い方](#基本的な使い方)
2. [AIプロンプト管理](#aiプロンプト管理)
3. [設定ファイル生成](#設定ファイル生成)
4. [ドキュメント生成](#ドキュメント生成)
5. [マルチ環境対応](#マルチ環境対応)
6. [高度なテクニック](#高度なテクニック)

## 基本的な使い方

### シンプルな挨拶文生成

**data.yaml:**
```yaml
name: 太郎
greeting: こんにちは
```

**template.txt:**
```
{{ greeting }}、{{ name }}さん！
```

**実行:**
```bash
rp -t template.txt -d data.yaml
```

**出力:**
```
こんにちは、太郎さん！
```

### 配列データの扱い

**data.yaml:**
```yaml
team: "開発チーム"
members:
  - name: "Alice"
    role: "リードエンジニア"
  - name: "Bob"
    role: "バックエンドエンジニア"
  - name: "Carol"
    role: "フロントエンドエンジニア"
```

**template.txt:**
```
# {{ team }}

リーダー: {{ members.0.name }} ({{ members.0.role }})
メンバー1: {{ members.1.name }} ({{ members.1.role }})
メンバー2: {{ members.2.name }} ({{ members.2.role }})
```

## AIプロンプト管理

### セグメント別プロンプト生成

ユーザーセグメントごとに異なるプロンプトを生成する例です。

**segments.yaml:**
```yaml
segments:
  engineer:
    title: "バックエンドエンジニア"
    description: "API設計とデータベース最適化が得意"
    pain_points:
      - "仕様変更が頻繁で疲弊している"
      - "技術的負債の解消に時間を割けない"
    motivations:
      - "技術的挑戦"
      - "裁量の大きさ"
      - "成長機会"
```

**appeals.yaml:**
```yaml
appeals:
  technical:
    headline: "技術的挑戦を楽しめる環境"
    body: |
      最新技術を積極的に採用し、技術的な裁量を持って
      プロダクトを作り上げていけます。
  growth:
    headline: "圧倒的な成長機会"
    body: |
      ドメイン知識とエンジニアリングスキルの両面で
      急速に成長できる環境です。
```

**partials/segment.txt:**
```
## ターゲットセグメント: {{ segments.engineer.title }}

### 特徴
{{ segments.engineer.description }}

### 課題
- {{ segments.engineer.pain_points.0 }}
- {{ segments.engineer.pain_points.1 }}

### 訴求ポイント
#### {{ appeals.technical.headline }}
{{ appeals.technical.body }}

#### {{ appeals.growth.headline }}
{{ appeals.growth.body }}
```

**prompt.txt:**
```
あなたは採用マーケティングのコピーライターです。

{{> partials/segment.txt }}

上記のセグメント情報を元に、魅力的な求人広告を作成してください。
```

**実行:**
```bash
rp \
  -t prompt.txt \
  -d segments.yaml \
  -d appeals.yaml \
  -o generated_prompt.txt
```

### プロンプトのバリエーション管理

**base_prompt.yaml:**
```yaml
role: "採用マーケティングのコピーライター"
task: "求人広告を作成"
format: "200文字以内の魅力的な文章"
```

**variant_a.yaml:**
```yaml
tone: "フレンドリー"
style: "カジュアル"
```

**variant_b.yaml:**
```yaml
tone: "プロフェッショナル"
style: "フォーマル"
```

**template.txt:**
```
あなたは{{ role }}です。

タスク: {{ task }}
形式: {{ format }}
トーン: {{ tone }}
スタイル: {{ style }}

以下の情報を元に作成してください：
...
```

**実行:**
```bash
# バリエーションA
rp -t template.txt -d base_prompt.yaml -d variant_a.yaml -o prompt_a.txt

# バリエーションB
rp -t template.txt -d base_prompt.yaml -d variant_b.yaml -o prompt_b.txt
```

## 設定ファイル生成

### Nginx設定ファイル

**config.yaml:**
```yaml
server:
  name: "example.com"
  port: 80
  root: "/var/www/html"

ssl:
  enabled: true
  cert: "/etc/ssl/certs/example.com.crt"
  key: "/etc/ssl/private/example.com.key"

locations:
  - path: "/"
    proxy_pass: "http://localhost:3000"
  - path: "/api"
    proxy_pass: "http://localhost:8080"
```

**nginx.conf.template:**
```
server {
    listen {{ server.port }};
    server_name {{ server.name }};
    root {{ server.root }};

    location {{ locations.0.path }} {
        proxy_pass {{ locations.0.proxy_pass }};
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    location {{ locations.1.path }} {
        proxy_pass {{ locations.1.proxy_pass }};
    }
}
```

**実行:**
```bash
rp -t nginx.conf.template -d config.yaml -o nginx.conf
```

### Docker Compose

**services.yaml:**
```yaml
app:
  name: "myapp"
  image: "node:18"
  port: 3000

db:
  name: "postgres"
  image: "postgres:15"
  port: 5432
  password: "secretpassword"
```

**docker-compose.yml.template:**
```yaml
version: '3.8'

services:
  {{ app.name }}:
    image: {{ app.image }}
    ports:
      - "{{ app.port }}:{{ app.port }}"
    environment:
      DATABASE_URL: postgresql://{{ db.name }}:{{ db.password }}@{{ db.name }}:{{ db.port }}/mydb
    depends_on:
      - {{ db.name }}

  {{ db.name }}:
    image: {{ db.image }}
    ports:
      - "{{ db.port }}:{{ db.port }}"
    environment:
      POSTGRES_PASSWORD: {{ db.password }}
```

## ドキュメント生成

### API ドキュメント

**api_spec.yaml:**
```yaml
api:
  name: "User Management API"
  version: "1.0.0"
  base_url: "https://api.example.com/v1"

endpoints:
  - method: "GET"
    path: "/users"
    description: "ユーザー一覧を取得"
    response: |
      [
        {"id": 1, "name": "Alice"},
        {"id": 2, "name": "Bob"}
      ]

  - method: "POST"
    path: "/users"
    description: "新規ユーザーを作成"
    request: |
      {
        "name": "Charlie",
        "email": "charlie@example.com"
      }
```

**api_docs.md.template:**
```markdown
# {{ api.name }}

バージョン: {{ api.version }}
ベースURL: {{ api.base_url }}

## エンドポイント

### {{ endpoints.0.method }} {{ endpoints.0.path }}

{{ endpoints.0.description }}

**レスポンス例:**
\```json
{{ endpoints.0.response }}
\```

### {{ endpoints.1.method }} {{ endpoints.1.path }}

{{ endpoints.1.description }}

**リクエスト例:**
\```json
{{ endpoints.1.request }}
\```
```

## マルチ環境対応

### 環境別設定の管理

**base.yaml:**
```yaml
app:
  name: "MyApp"
  log_level: "info"

features:
  analytics: true
  debug_mode: false
```

**dev.yaml:**
```yaml
app:
  log_level: "debug"

features:
  debug_mode: true

database:
  host: "localhost"
  port: 5432
```

**prod.yaml:**
```yaml
app:
  log_level: "warn"

features:
  analytics: true

database:
  host: "prod-db.example.com"
  port: 5432
```

**config.template:**
```ini
[app]
name = {{ app.name }}
log_level = {{ app.log_level }}

[features]
analytics = {{ features.analytics }}
debug_mode = {{ features.debug_mode }}

[database]
host = {{ database.host }}
port = {{ database.port }}
```

**実行:**
```bash
# 開発環境
rp -t config.template -d base.yaml -d dev.yaml -o config.dev.ini

# 本番環境
rp -t config.template -d base.yaml -d prod.yaml -o config.prod.ini
```

## 高度なテクニック

### 部分テンプレートの再利用

**partials/header.txt:**
```
=====================================
{{ title }}
=====================================
作成日: {{ date }}
```

**partials/footer.txt:**
```
-------------------------------------
{{ copyright }}
```

**document.txt:**
```
{{> partials/header.txt }}

{{ content }}

{{> partials/footer.txt }}
```

**data.yaml:**
```yaml
title: "月次レポート"
date: "2024-01-15"
content: |
  ## 成果
  - 目標達成率: 120%
  - 新規顧客: 50社

  ## 課題
  - リソース不足
  - スケジュール遅延
copyright: "© 2024 MyCompany"
```

### エスケープの活用

テンプレート構文をそのまま出力したい場合：

**data.yaml:**
```yaml
tool_name: "rp"
example_syntax: "変数を埋め込む"
```

**tutorial.md.template:**
```markdown
# {{ tool_name }} チュートリアル

## 変数置換の方法

{{ example_syntax }}には、\{{ variable }} という構文を使います。

例:
\```
名前: \{{ name }}
年齢: \{{ age }}
\```

これを実行すると、実際の値が埋め込まれます。
```

### データなしでのテンプレート使用

データファイルを指定しない場合、全ての変数が空文字になります：

**template.txt:**
```
Hello, {{ name }}!

\{{ name }} には実際の名前が入ります。
```

**実行:**
```bash
rp -t template.txt
```

**出力:**
```
Hello, !

{{ name }} には実際の名前が入ります。
```

### Strict モードでの開発

開発時は`--strict`モードを使うことで、未定義変数を早期発見できます：

```bash
# 未定義変数があるとエラーで終了
rp -t template.txt -d data.yaml --strict
```

これにより、タイポや設定漏れを防げます。

## ベストプラクティス

### 1. ディレクトリ構成

```
project/
├── templates/
│   ├── base.txt
│   └── partials/
│       ├── header.txt
│       └── footer.txt
├── data/
│   ├── base.yaml
│   ├── dev.yaml
│   └── prod.yaml
└── output/
    ├── dev/
    └── prod/
```

### 2. 命名規則

- テンプレートファイル: `*.template.txt` または `*.txt.template`
- データファイル: 環境名を明記 `dev.yaml`, `prod.yaml`
- 部分テンプレート: `partials/` ディレクトリに配置

### 3. バージョン管理

- テンプレートとデータファイルはGit管理
- 生成されたファイルは`.gitignore`に追加
- データファイルに秘密情報を含めない

### 4. CI/CD統合

```bash
# Makefileの例
.PHONY: generate-dev generate-prod

generate-dev:
	rp -t templates/config.template \
	  -d data/base.yaml \
	  -d data/dev.yaml \
	  -o output/dev/config.ini

generate-prod:
	rp -t templates/config.template \
	  -d data/base.yaml \
	  -d data/prod.yaml \
	  -o output/prod/config.ini
```

## トラブルシューティング

### 変数が置換されない

**原因**: タイポや未定義変数

**解決策**: `--strict`または`--warn-undefined`を使用

```bash
rp -t template.txt -d data.yaml --warn-undefined
```

### インクルードが見つからない

**原因**: 相対パスが間違っている

**解決策**: `--root`オプションでルートディレクトリを指定

```bash
rp -t template.txt -d data.yaml --root ./templates
```

### 循環インクルードエラー

**原因**: A → B → A のような循環参照

**解決策**: インクルード構造を見直す

## まとめ

rpは、シンプルながら強力なテンプレートツールです。
複雑なロジックは持たないため、データの準備が重要になります。

より詳細な情報は[README.md](README.md)を参照してください。
