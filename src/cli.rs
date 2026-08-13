use argh::FromArgs;

#[derive(FromArgs)]
#[argh(description = "AI Git Commit 助手")]
pub struct Cli {
    #[argh(positional, description = "目录匹配模式，从历史记录中过滤")]
    pub directory: Option<String>,

    #[argh(option, description = "提交完成后执行 git push 到指定远程仓库 (例如: --push origin)")]
    pub push: Option<String>,

    #[argh(option, description = "仅执行 git pull 到指定远程仓库 (例如: --pull origin)，不进行 AI 提交")]
    pub pull: Option<String>,

    #[argh(option, description = "path to git-commit SKILL.md")]
    pub skill_path: Option<String>,
    #[argh(option, description = "API key (优先级高于 MESSAGE_API_KEY 环境变量)")]
    pub api_key: Option<String>,
    #[argh(option, description = "rg 搜索模式，不指定则自动检测代码结构")]
    pub rg_pattern: Option<String>,
    #[argh(option, description = "fd 搜索模式，查找关联文件")]
    pub fd_pattern: Option<String>,
    #[argh(option, description = "lazygit 配置路径（默认 .lazygit.yaml）")]
    pub lazygit_config: Option<String>,

    #[argh(option, description = "AI 配置目录（默认 ~/.config/agc，含 default.yaml）")]
    pub config_dir: Option<String>,

    #[argh(switch, short = 'y', description = "跳过人工确认，生成提交消息后直接提交")]
    pub yes: bool,

    #[argh(subcommand)]
    pub sub: Option<SubCommands>,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum SubCommands {
    Config(ConfigArgs),
    Git(GitArgs),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "config")]
/// 配置管理
pub struct ConfigArgs {
    #[argh(subcommand)]
    pub sub: ConfigSubCommands,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum ConfigSubCommands {
    Init(InitArgs),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "init")]
/// 在指定目录初始化 AI 配置文件 default.yaml
pub struct InitArgs {
    /// 配置目录（默认 ~/.config/agc）
    #[argh(positional)]
    pub path: Option<String>,

    /// 覆盖已存在的配置文件
    #[argh(switch)]
    pub force: bool,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "git")]
/// Git 仓库管理
pub struct GitArgs {
    #[argh(subcommand)]
    pub sub: GitSubCommands,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum GitSubCommands {
    Init(GitInitArgs),
    Update(GitUpdateArgs),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "init")]
/// 初始化 git 仓库与 .project 目录
pub struct GitInitArgs {
    /// 覆盖已存在的 .git-style-scope.yaml
    #[argh(switch)]
    pub force: bool,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "update")]
/// 根据项目结构与 git 历史更新 .git-style-scope.yaml
pub struct GitUpdateArgs {
    /// 删除已不在项目结构中的 scope
    #[argh(switch)]
    pub prune: bool,

    /// AI 配置目录（默认 ~/.config/agc）
    #[argh(option)]
    pub config_dir: Option<String>,

    /// API key（默认读取 MESSAGE_API_KEY 环境变量）
    #[argh(option)]
    pub api_key: Option<String>,
}

impl Cli {
    pub fn parse() -> Self {
        argh::from_env()
    }
}
