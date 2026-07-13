use clap::Parser;
use agc::app::{self, Workflow};

#[derive(Parser)]
#[command(name = "agc", about = "AI Git Commit 助手")]
struct Cli {
    #[arg(long, help = "path to git-commit SKILL.md")]
    skill_path: Option<String>,
    #[arg(long, help = "API key (优先级高于 MESSAGE_API_KEY 环境变量)")]
    api_key: Option<String>,
    #[arg(long, help = "rg 搜索模式，不指定则自动检测代码结构")]
    rg_pattern: Option<String>,
    #[arg(long, help = "fd 搜索模式，查找关联文件")]
    fd_pattern: Option<String>,
    #[arg(long, help = "lazygit 配置路径（默认 .lazygit.yaml）")]
    lazygit_config: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let skill_path = cli.skill_path.unwrap_or_default();
    let api_key = cli.api_key.unwrap_or_default();
    let rg_pattern = cli.rg_pattern.unwrap_or_default();
    let fd_pattern = cli.fd_pattern.unwrap_or_default();
    let lazygit_path = cli.lazygit_config.unwrap_or_default();

    let wf = Workflow::new();
    wf.run(app::Options {
        skill_path,
        api_key,
        rg_pattern,
        fd_pattern,
        lazygit_path,
    })
    .await
}
