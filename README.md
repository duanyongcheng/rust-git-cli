# Rust Git CLI

[English](README.en.md) | [日本語](README.ja.md) | [Deutsch](README.de.md) | 中文

一个智能的 Git 提交工具，使用 AI 生成中英文双语提交信息。

An intelligent Git commit tool that generates bilingual (Chinese/English) commit messages using AI.

## 特性 Features

- **AI 驱动** - 支持 OpenAI、Anthropic 等 AI 提供商，可配置自定义端点（如 DeepSeek）
- **双语提交** - 自动生成符合 Conventional Commits 规范的中英文双语提交信息
- **智能暂存** - 自动检测未暂存更改并提示确认
- **交互式界面** - 彩色输出、差异预览、提交确认
- **AI Changelog** - 交互式选择提交记录，AI 生成 changelog 总结，支持复制到剪切板
- **灵活配置** - 支持多级配置文件和环境变量

## 安装 Installation

```bash
# 从源码编译
git clone https://github.com/duanyongcheng/rust-git-cli.git
cd rust-git-cli
cargo build --release

# 安装到系统
cargo install --path .
```

**系统要求**: Rust 1.70+, Git 2.0+

## 快速开始 Quick Start

### 1. 初始化配置

```bash
rust-git-cli init                 # 创建全局配置 (~/.config/rust-git-cli/config.toml)
rust-git-cli init --local         # 创建项目配置 (.rust-git-cli.toml)
```

### 2. 设置 API Key

```bash
# 推荐：使用环境变量
export OPENAI_API_KEY="your-api-key"
# 或
export ANTHROPIC_API_KEY="your-api-key"
```

### 3. 使用

```bash
rust-git-cli                      # 查看仓库状态（默认命令）
rust-git-cli commit               # AI 生成提交信息
rust-git-cli commit --show-diff   # 预览差异后生成
rust-git-cli commit --debug       # 调试模式
```

## 命令 Commands

| 命令 | 说明 |
|------|------|
| `status` | 查看仓库状态（默认） |
| `commit` | AI 生成并执行提交 |
| `diff` | 查看代码差异 |
| `log` | 查看提交历史，支持 AI 生成 changelog |
| `init` | 初始化配置文件 |

### commit 命令选项

```bash
rust-git-cli commit [OPTIONS]

Options:
  --api-key <KEY>      临时指定 API Key
  --model <MODEL>      指定 AI 模型 (如 gpt-4, deepseek-v3)
  --base-url <URL>     自定义 API 端点
  --auto               跳过确认直接提交
  --show-diff          生成前预览差异
  --debug              显示 AI 原始响应
```

### log 命令选项

```bash
rust-git-cli log [OPTIONS]

Options:
  -n, --count <N>      显示条数 (默认 10)
  --grep <PATTERN>     按内容过滤
  --author <NAME>      按作者过滤
  --since <DATE>       起始日期 (如 "2024-01-01" 或 "1 week ago")
  --until <DATE>       截止日期
  --full               显示完整提交信息
  --api-key <KEY>      临时指定 API Key (用于生成 changelog)
  --model <MODEL>      指定 AI 模型
  --base-url <URL>     自定义 API 端点
  --debug              显示 AI 原始响应
```

### diff 命令选项

```bash
rust-git-cli diff [OPTIONS]

Options:
  --staged             仅显示已暂存的更改
```

## 配置 Configuration

配置文件查找顺序：
1. `./.rust-git-cli.toml` (项目级)
2. `~/.config/rust-git-cli/config.toml` (用户级)
3. `~/.rust-git-cli.toml` (用户级备选)

### 配置示例

```toml
[ai]
provider = "openai"                      # openai 或 anthropic
model = "gpt-4"                          # 模型名称
api_key_env = "OPENAI_API_KEY"           # API Key 环境变量名
# api_key = "sk-..."                     # 直接设置 (不推荐)
# base_url = "https://api.deepseek.com/v1"  # 自定义端点
max_tokens = 2000

[commit]
max_diff_size = 4000                     # 发送给 AI 的最大差异字符数
auto_stage = false                       # 是否自动暂存所有更改
```

### API Key 优先级

1. 命令行参数 `--api-key`
2. 配置文件 `api_key`
3. 环境变量 (由 `api_key_env` 指定)
4. 交互式输入

## 提交信息格式 Commit Format

生成的提交信息遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
feat(auth): 添加用户认证功能
Add user authentication feature

实现了JWT令牌验证
Implement JWT token validation
添加了用户登录接口
Add user login endpoint
```

### 提交类型

| 类型 | 说明 |
|------|------|
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `docs` | 文档更新 |
| `style` | 代码格式 |
| `refactor` | 代码重构 |
| `test` | 测试相关 |
| `chore` | 构建/工具 |
| `perf` | 性能优化 |

## 工作流程 Workflow

```bash
# 1. 查看状态
$ rust-git-cli status

# 2. 查看差异
$ rust-git-cli diff

# 3. 生成提交
$ rust-git-cli commit

# 检测到未暂存更改时会提示：
# Unstaged changes detected:
# ──────────────────────────────────────────────────
#   M src/main.rs
#   ? src/new_file.rs
# ──────────────────────────────────────────────────
# Do you want to stage all changes (git add .)? (Y/n)

# AI 生成后选择操作：
# - Accept and commit: 接受并提交
# - Edit message: 编辑后提交
# - Regenerate: 重新生成
# - Cancel: 取消

# 4. 生成 Changelog
$ rust-git-cli log -n 20

# 交互式选择提交记录 (Space 选择, Enter 确认)
# 选择后可生成 AI changelog 总结
# 支持复制到剪切板
```

## 故障排除 Troubleshooting

| 问题 | 解决方案 |
|------|----------|
| API 连接失败 | 检查网络、验证 API Key、使用 `--debug` 查看详情 |
| JSON 解析错误 | 使用 `--debug` 查看原始响应，尝试更换模型 |
| 配置未生效 | 检查文件路径和 TOML 格式 |
| 提交失败 | 确认 Git 用户已配置 (`git config user.name/email`) |

```bash
# 调试模式
rust-git-cli commit --debug

# 查看帮助
rust-git-cli --help
rust-git-cli commit --help
```

## 开发 Development

```bash
cargo build                       # 构建
cargo test                        # 测试
cargo fmt                         # 格式化
cargo clippy -- -D warnings       # Lint
```

### 项目结构

```
src/
├── main.rs          # 入口和命令分发
├── cli.rs           # 命令行定义 (clap)
├── config.rs        # 配置管理
├── git.rs           # Git 操作 (git2)
├── ui.rs            # 交互界面 (dialoguer)
└── ai/
    ├── mod.rs       # AI 客户端抽象
    ├── openai.rs    # OpenAI 实现
    └── anthropic.rs # Anthropic 实现
```

## 许可证 License

MIT License

## 致谢 Credits

- [git2-rs](https://github.com/rust-lang/git2-rs) - Git 操作
- [clap](https://github.com/clap-rs/clap) - 命令行解析
- [dialoguer](https://github.com/console-rs/dialoguer) - 交互式界面
- [colored](https://github.com/colored-rs/colored) - 彩色输出
