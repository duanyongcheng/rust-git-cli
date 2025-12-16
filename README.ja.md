# Rust Git CLI

AIを使用して中国語/英語のバイリンガルコミットメッセージを生成するインテリジェントなGitコミットツール。

## 特徴

- **AI駆動** - OpenAI、Anthropic、カスタムエンドポイント（DeepSeekなど）をサポート
- **バイリンガルコミット** - Conventional Commits規約に従った中国語/英語のコミットメッセージを自動生成
- **スマートステージング** - ステージされていない変更を検出し、確認を求める
- **インタラクティブUI** - カラー出力、差分プレビュー、コミット確認
- **柔軟な設定** - マルチレベルの設定ファイルと環境変数

## インストール

```bash
# ソースからビルド
git clone https://github.com/duanyongcheng/rust-git-cli.git
cd rust-git-cli
cargo build --release

# システムにインストール
cargo install --path .
```

**要件**: Rust 1.70+, Git 2.0+

## クイックスタート

### 1. 設定の初期化

```bash
rust-git-cli init                 # グローバル設定を作成 (~/.config/rust-git-cli/config.toml)
rust-git-cli init --local         # プロジェクト設定を作成 (.rust-git-cli.toml)
```

### 2. APIキーの設定

```bash
# 推奨：環境変数を使用
export OPENAI_API_KEY="your-api-key"
# または
export ANTHROPIC_API_KEY="your-api-key"
```

### 3. 使用方法

```bash
rust-git-cli                      # リポジトリの状態を確認（デフォルト）
rust-git-cli commit               # AIコミットメッセージを生成
rust-git-cli commit --show-diff   # 生成前に差分をプレビュー
rust-git-cli commit --debug       # デバッグモード
```

## コマンド

| コマンド | 説明 |
|---------|------|
| `status` | リポジトリの状態を確認（デフォルト） |
| `commit` | AIコミットを生成して実行 |
| `diff` | コード変更を表示 |
| `log` | コミット履歴を表示 |
| `init` | 設定ファイルを初期化 |

### commit オプション

```bash
rust-git-cli commit [OPTIONS]

Options:
  --api-key <KEY>      APIキーを一時的に指定
  --model <MODEL>      AIモデルを指定（例：gpt-4, deepseek-v3）
  --base-url <URL>     カスタムAPIエンドポイント
  --auto               確認をスキップして直接コミット
  --show-diff          生成前に差分をプレビュー
  --debug              AIの生レスポンスを表示
```

### log オプション

```bash
rust-git-cli log [OPTIONS]

Options:
  -n, --count <N>      表示するコミット数（デフォルト：10）
  --grep <PATTERN>     内容でフィルタ
  --author <NAME>      作者でフィルタ
  --since <DATE>       開始日（例："2024-01-01" または "1 week ago"）
  --until <DATE>       終了日
  --full               完全なコミットメッセージを表示
```

### diff オプション

```bash
rust-git-cli diff [OPTIONS]

Options:
  --staged             ステージされた変更のみ表示
```

## 設定

設定ファイルの検索順序：
1. `./.rust-git-cli.toml`（プロジェクトレベル）
2. `~/.config/rust-git-cli/config.toml`（ユーザーレベル）
3. `~/.rust-git-cli.toml`（ユーザーレベルのフォールバック）

### 設定例

```toml
[ai]
provider = "openai"                      # openai または anthropic
model = "gpt-4"                          # モデル名
api_key_env = "OPENAI_API_KEY"           # APIキー環境変数名
# api_key = "sk-..."                     # 直接設定（非推奨）
# base_url = "https://api.deepseek.com/v1"  # カスタムエンドポイント
max_tokens = 2000

[commit]
max_diff_size = 4000                     # AIに送信する最大差分文字数
auto_stage = false                       # すべての変更を自動ステージ
```

### APIキーの優先順位

1. コマンドライン引数 `--api-key`
2. 設定ファイル `api_key`
3. 環境変数（`api_key_env`で指定）
4. 対話式入力

## コミットメッセージの形式

生成されるコミットメッセージは[Conventional Commits](https://www.conventionalcommits.org/)規約に従います：

```
feat(auth): 添加用户认证功能
Add user authentication feature

实现了JWT令牌验证
Implement JWT token validation
添加了用户登录接口
Add user login endpoint
```

### コミットタイプ

| タイプ | 説明 |
|--------|------|
| `feat` | 新機能 |
| `fix` | バグ修正 |
| `docs` | ドキュメント |
| `style` | コードフォーマット |
| `refactor` | リファクタリング |
| `test` | テスト |
| `chore` | ビルド/ツール |
| `perf` | パフォーマンス |

## ワークフロー

```bash
# 1. 状態を確認
$ rust-git-cli status

# 2. 変更を表示
$ rust-git-cli diff

# 3. コミットを生成
$ rust-git-cli commit

# ステージされていない変更が検出された場合：
# Unstaged changes detected:
# ──────────────────────────────────────────────────
#   M src/main.rs
#   ? src/new_file.rs
# ──────────────────────────────────────────────────
# Do you want to stage all changes (git add .)? (Y/n)

# AI生成後、アクションを選択：
# - Accept and commit: 承認してコミット
# - Edit message: 編集してからコミット
# - Regenerate: 再生成
# - Cancel: キャンセル
```

## トラブルシューティング

| 問題 | 解決策 |
|------|--------|
| API接続失敗 | ネットワークを確認、APIキーを検証、`--debug`で詳細を確認 |
| JSON解析エラー | `--debug`で生レスポンスを確認、別のモデルを試す |
| 設定が機能しない | ファイルパスとTOML形式を確認 |
| コミット失敗 | Gitユーザーが設定されていることを確認（`git config user.name/email`） |

```bash
# デバッグモード
rust-git-cli commit --debug

# ヘルプを表示
rust-git-cli --help
rust-git-cli commit --help
```

## 開発

```bash
cargo build                       # ビルド
cargo test                        # テスト
cargo fmt                         # フォーマット
cargo clippy -- -D warnings       # リント
```

### プロジェクト構造

```
src/
├── main.rs          # エントリーポイントとコマンドディスパッチ
├── cli.rs           # CLI定義（clap）
├── config.rs        # 設定管理
├── git.rs           # Git操作（git2）
├── ui.rs            # インタラクティブUI（dialoguer）
└── ai/
    ├── mod.rs       # AIクライアント抽象化
    ├── openai.rs    # OpenAI実装
    └── anthropic.rs # Anthropic実装
```

## ライセンス

MIT License

## クレジット

- [git2-rs](https://github.com/rust-lang/git2-rs) - Git操作
- [clap](https://github.com/clap-rs/clap) - コマンドライン解析
- [dialoguer](https://github.com/console-rs/dialoguer) - インタラクティブUI
- [colored](https://github.com/colored-rs/colored) - カラー出力
