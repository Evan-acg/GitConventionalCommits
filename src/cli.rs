use argh::FromArgs;

#[derive(FromArgs)]
#[argh(description = "AI Git Commit 助手")]
pub struct Cli {
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
    Push(PushArgs),
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

#[derive(FromArgs)]
#[argh(subcommand, name = "push")]
/// 在指定目录执行 AI 提交流程并推送到远程仓库
pub struct PushArgs {
    /// 目录匹配模式，从历史记录中过滤（省略则在当前目录运行）
    #[argh(option, short = 't')]
    pub target: Option<String>,

    /// 推送分支（默认 master）
    #[argh(option, short = 'b')]
    pub branch: Option<String>,

    /// 远程仓库（默认 origin）
    #[argh(option, short = 'r')]
    pub remote: Option<String>,

    /// path to git-commit SKILL.md
    #[argh(option)]
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
}

/// push 子命令的默认值
pub const DEFAULT_REMOTE: &str = "origin";
pub const DEFAULT_BRANCH: &str = "master";

impl PushArgs {
    pub fn remote(&self) -> &str {
        self.remote.as_deref().unwrap_or(DEFAULT_REMOTE)
    }

    pub fn branch(&self) -> &str {
        self.branch.as_deref().unwrap_or(DEFAULT_BRANCH)
    }
}

impl Cli {
    pub fn parse() -> Self {
        argh::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use argh::FromArgs;

    fn parse_push(args: &[&str]) -> PushArgs {
        let mut all = vec!["push"];
        all.extend_from_slice(args);
        let cli = Cli::from_args(&["agc"], &all).expect("push 子命令解析失败");
        match cli.sub {
            Some(SubCommands::Push(push)) => push,
            _ => panic!("未解析出 push 子命令"),
        }
    }

    #[test]
    fn parse_push_full_args() {
        let args = parse_push(&["-t", "rime", "-b", "dev", "-r", "upstream", "-y"]);
        assert_eq!(args.target.as_deref(), Some("rime"));
        assert_eq!(args.branch(), "dev");
        assert_eq!(args.remote(), "upstream");
        assert!(args.yes);
    }

    #[test]
    fn parse_push_defaults() {
        let args = parse_push(&["-t", "rime"]);
        assert_eq!(args.target.as_deref(), Some("rime"));
        assert_eq!(args.branch(), DEFAULT_BRANCH);
        assert_eq!(args.remote(), DEFAULT_REMOTE);
        assert!(!args.yes);
    }

    #[test]
    fn parse_push_without_target() {
        let args = parse_push(&[]);
        assert!(args.target.is_none());
        assert_eq!(args.branch(), DEFAULT_BRANCH);
        assert_eq!(args.remote(), DEFAULT_REMOTE);
    }

    #[test]
    fn parse_long_options() {
        let args = parse_push(&["--target", "rime", "--branch", "dev", "--remote", "upstream"]);
        assert_eq!(args.target.as_deref(), Some("rime"));
        assert_eq!(args.branch(), "dev");
        assert_eq!(args.remote(), "upstream");
    }
}
