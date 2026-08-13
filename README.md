# agc — AI Git Commit 助手

> Rust 版：基于 AI 的 Conventional Commits 提交信息生成工具。

`agc` 是一款用 Rust 编写的命令行工具。它会分析当前 Git 仓库的变更（diff），
结合代码结构与仓库上下文，调用 AI 生成符合 Conventional Commits 规范的提交信息，
并支持一键提交与推送。

## 特性

- **AI 生成提交信息**：基于 git diff 自动生成 `Feat(Scope): 描述` 格式的提交信息
- **代码上下文增强**：使用 `rg` / `fd` 分析代码结构，找到关联文件，让 AI 更准确
- **目录历史与快速切换**：记录使用过的目录（LRU），支持模糊匹配与 fzf 选择
- **OpenAI 兼容协议**：支持 DeepSeek、OpenAI 及任何 Chat Completions 兼容端点
- **多配置源链式合并**：type/scope 与 AI 配置均支持多来源按优先级合并
- **快速操作**：`--push` 提交后自动推送、`--pull` 仅拉取、`-y` 免确认提交
- **交互提示**：彩色输出、spinner 加载动画、生成后可人工确认

## 工作原理

`agc` 以**管线（Pipeline）**方式按序执行各个阶段，每个阶段对上下文做一次转换：

```
DiffCollector → ContextEnricher → AiGenerator → EntryParser → CommitExecutor → [GitPusher]
```

| 阶段 | 职责 |
| --- | --- |
| `DiffCollector` | 收集 git diff 与仓库信息（`git`） |
| `ContextEnricher` | 用 `rg`/`fd` 搜索代码结构，丰富 AI 上下文 |
| `AiGenerator` | 调用 AI 生成候选提交信息（spinner 提示） |
| `EntryParser` | 解析 AI 返回的 JSON 响应 |
| `CommitExecutor` | 执行 `git commit`（生成后人工确认） |
| `GitPusher` | （可选，`--push` 时启用）执行 `git push` |

## 安装

### 从源码构建

```bash
cargo build --release
```

构建产物位于 `target/release/agc.exe`（Windows）/ `target/release/agc`（Unix）。

### 构建并部署（Windows）

```powershell
# 构建 release 并部署到 C:\Tools\agc.exe
.\build\deploy.ps1

# 仅构建，不部署
.\build\deploy.ps1 -NoDeploy

# 跳过构建，仅部署已有产物
.\build\deploy.ps1 -SkipBuild
```

Unix 环境可使用 `build/deploy.sh`。

### 外部依赖

| 工具 | 用途 | 是否必需 |
| --- | --- | --- |
| `git` | 收集 diff、执行提交/推送 | 必需 |
| `rg` (ripgrep) | 自动检测代码结构 | 可选（未安装时跳过） |
| `fd` | 查找关联文件 | 可选（未安装时跳过） |
| `fzf` | 多目录匹配时选择 | 可选（未安装时回退为序号输入） |

## 快速开始

```bash
# 1. 初始化 AI 配置文件（生成 ~/.config/agc/default.yaml）
agc config init

# 2. 编辑配置文件，填入 API Key（或使用环境变量 MESSAGE_API_KEY）
notepad $env:USERPROFILE\.config\agc\default.yaml   # Windows
# vim ~/.config/agc/default.yaml                    # Unix

# 3. 在 Git 仓库中运行
agc
```

运行流程：选择/记录目录 → 收集变更 → AI 生成提交信息 → 人工确认 → 提交。
`-y` 可跳过确认直接提交：

```bash
agc -y --push origin
```

## 命令行参数

```text
Usage: agc [--push <push>] [--pull <pull>] [--skill-path <skill-path>]
           [--api-key <api-key>] [--rg-pattern <rg-pattern>]
           [--fd-pattern <fd-pattern>] [--lazygit-config <lazygit-config>]
           [--config-dir <config-dir>] [-y] [<directory>] [<command>] [<args>]
```

| 参数 | 说明 |
| --- | --- |
| `<directory>` | 目录匹配模式，从历史记录中过滤（省略则在当前目录运行并记录） |
| `--push <remote>` | 提交完成后执行 `git push`（如 `--push origin`） |
| `--pull <remote>` | 仅执行 `git pull`，不进行 AI 提交 |
| `--skill-path` | git-commit 技能文档 SKILL.md 路径 |
| `--api-key` | API Key（优先级高于 `MESSAGE_API_KEY` 环境变量） |
| `--rg-pattern` | rg 搜索模式，不指定则自动检测代码结构 |
| `--fd-pattern` | fd 搜索模式，查找关联文件 |
| `--lazygit-config` | lazygit 配置路径（默认 `.lazygit.yaml`） |
| `--config-dir` | AI 配置目录（默认 `~/.config/agc`，含 `default.yaml`） |
| `-y, --yes` | 跳过人工确认，生成提交信息后直接提交 |

### 子命令

```bash
# 在指定目录初始化 AI 配置文件 default.yaml
agc config init [path] [--force]
```

- `path`：配置目录（默认 `~/.config/agc`）
- `--force`：覆盖已存在的配置文件

```bash
# 在当前目录初始化 git 仓库（已初始化则跳过），并生成 .project/.git-style-scope.yaml
agc git init [--force]

# 根据项目结构与 git 历史更新 .git-style-scope.yaml（由 LLM 生成 scope 列表）
agc git update [--prune] [--config-dir <目录>] [--api-key <key>]
```

`.git-style-scope.yaml` 查找优先级（1 > 2 > 3）：`.project/` 目录 > 项目根目录 > 家目录；`update` 读取按此顺序，写入始终落在 `.project/`。

- `--force`：覆盖已存在的 `.project/.git-style-scope.yaml`
- `--prune`：删除已不在项目结构中的 scope（默认只增不删）
- `--config-dir` / `--api-key`：AI 配置目录与 API key（默认 `~/.config/agc` 与 `MESSAGE_API_KEY`）

## 配置

### type / scope 配置链（按优先级）

1. **`.lazygit.yaml`**（或 `--lazygit-config` 指定）— 格式见 `docs/.lazygit.yaml`
2. **SKILL.md**（`--skill-path` 指定）— 从 Markdown 表格解析 type（如 ``| `feat` | 新特性 |``）
3. **内置默认值** — `Feat` / `Fix` / `Docs` / `Style` / `Refactor` / `Perf` / `Test` / `Build` / `CI` / `Chore` / `Revert`

`.lazygit.yaml` 示例：

```yaml
type:
  - name: Feat
    docs: 新特性
  - name: Fix
    docs: 修补校验
scope:
  - name: Core
    docs: 核心模块
  - name: UI
    docs: 界面
```

### AI 配置链（按优先级）

1. **配置文件** `~/.config/agc/default.yaml`（或 `--config-dir` 指定）
2. **环境变量**：`MESSAGE_API_KEY`、`AI_PROVIDER`、`OPENAI_MODEL`、`OPENAI_BASE_URL`
3. **内置默认值**：provider `openai`，模型 `deepseek-v4-flash`

`default.yaml` 示例（可由 `agc config init` 生成）：

```yaml
# use_provider 选择 providers 列表中要使用的连接配置
use_provider: deepseek
providers:
  - name: deepseek
    # kind 为协议实现类型（ProviderRegistry 注册名），默认 openai
    kind: openai
    # base_url 为完整 chat/completions 端点地址
    base_url: https://api.deepseek.com/v1/chat/completions
    model: deepseek-v4-flash
    # api_key 可省略，省略时回退到 MESSAGE_API_KEY 环境变量
    api_key: sk-xxx
```

也兼容旧版扁平结构（`provider` / `api_key` / `model` / `base_url`），加载时会自动归一化。
API Key 最终优先级：`--api-key` > 配置文件 > `MESSAGE_API_KEY` 环境变量。

## 目录历史

`agc` 会记录使用过的目录，按 LRU（最近使用）排序：

- 存储位置：XDG 规范路径 `~/.local/share/agc/history`，回退 `~/.agc_history`
- 传入 `<directory>` 参数时按子串模糊匹配（大小写不敏感）
- 匹配多个目录时优先用 `fzf` 选择，未安装则回退为序号输入

## 测试

```bash
# 单元测试
cargo test --lib

# 集成测试（cucumber BDD，基于 wiremock 模拟 API）
cargo test --test agc
```

## 项目结构

```text
src/
├── main.rs            # 入口：参数解析、目录选择、配置加载、管线组装
├── cli.rs             # argh 命令行参数定义
├── ai/                # AI 提供方（openai 协议）、Prompt 构建、ProviderRegistry
├── commit/            # 提交执行与消息解析
├── config/            # 配置门面 + 多源链式合并（yaml / markdown / 默认值 / 环境变量）
├── git/               # Git 后端（real 实现，trait 化便于测试）
├── history.rs         # 目录历史（LRU、模糊过滤、fzf 选择）
├── pipeline/          # 管线架构：builder + 各阶段（diff/enrich/ai/parse/commit/push）
├── search/            # 代码搜索后端（rg / fd）
├── strutil.rs         # 字符串工具
└── ui/                # 彩色输出与 spinner
tests/                 # cucumber BDD 集成测试
build/                 # 构建与部署脚本
docs/                  # 示例配置
```
