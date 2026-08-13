use std::sync::Arc;

use agc::cli::{Cli, ConfigSubCommands, GitSubCommands, PushArgs, SubCommands};
use agc::config::AppConfig;
use agc::git::real::RealGit;
use agc::git::GitBackend;
use agc::history::History;
use agc::pipeline::{PipelineBuilder, PipelineContext};
use agc::search::RealSearch;
use agc::ui::color;

/// 完整提交流程的输入参数（裸 agc 与 push 子命令共用）
struct FlowOptions {
    auto_push: Option<(String, String)>,
    auto_commit: bool,
    skill_path: Option<String>,
    api_key: Option<String>,
    rg_pattern: Option<String>,
    fd_pattern: Option<String>,
    lazygit_config: Option<String>,
    config_dir: Option<String>,
}

impl FlowOptions {
    fn from_cli(cli: &Cli) -> Self {
        Self {
            auto_push: None,
            auto_commit: cli.yes,
            skill_path: cli.skill_path.clone(),
            api_key: cli.api_key.clone(),
            rg_pattern: cli.rg_pattern.clone(),
            fd_pattern: cli.fd_pattern.clone(),
            lazygit_config: cli.lazygit_config.clone(),
            config_dir: cli.config_dir.clone(),
        }
    }

    fn from_push(args: &PushArgs) -> Self {
        Self {
            auto_push: Some((args.remote().to_string(), args.branch().to_string())),
            auto_commit: args.yes,
            skill_path: args.skill_path.clone(),
            api_key: args.api_key.clone(),
            rg_pattern: args.rg_pattern.clone(),
            fd_pattern: args.fd_pattern.clone(),
            lazygit_config: args.lazygit_config.clone(),
            config_dir: args.config_dir.clone(),
        }
    }
}

fn main() {
    let cli = Cli::parse();

    // ── 步骤0：config 子命令（配置管理） ──
    if let Some(SubCommands::Config(config)) = &cli.sub {
        match &config.sub {
            ConfigSubCommands::Init(args) => {
                match agc::config::init::run(args.path.clone(), args.force) {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("{}", color::red(&e.to_string()));
                        std::process::exit(1);
                    }
                }
            }
        }
        return;
    }

    // ── 步骤0：git 子命令（仓库管理） ──
    if let Some(SubCommands::Git(git)) = &cli.sub {
        match &git.sub {
            GitSubCommands::Init(args) => {
                match agc::git::init::run(args.force) {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("{}", color::red(&e.to_string()));
                        std::process::exit(1);
                    }
                }
            }
            GitSubCommands::Update(args) => {
                match agc::git::update::run(
                    args.prune,
                    args.config_dir.clone(),
                    args.api_key.clone(),
                ) {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("{}", color::red(&e.to_string()));
                        std::process::exit(1);
                    }
                }
            }
        }
        return;
    }

    // ── 步骤0：push 子命令（选目录 → 完整流程 → 推送） ──
    if let Some(SubCommands::Push(args)) = &cli.sub {
        if let Err(e) = select_directory(args.target.as_deref()) {
            eprintln!("{}", color::red(&e.to_string()));
            std::process::exit(1);
        }
        run_flow(FlowOptions::from_push(args));
        return;
    }

    // ── 裸 agc：当前目录，提交但不推送 ──
    if let Err(e) = select_directory(None) {
        eprintln!("{}", color::red(&e.to_string()));
        std::process::exit(1);
    }
    run_flow(FlowOptions::from_cli(&cli));
}

/// 目录选择：有 pattern 时从历史记录过滤并切换目录，无 pattern 时记录当前目录
fn select_directory(pattern: Option<&str>) -> anyhow::Result<()> {
    match pattern {
        Some(pattern) => {
            let history = History::load();
            let matches = history.filter(pattern);
            match matches.len() {
                0 => anyhow::bail!("没有匹配的历史目录: {pattern}"),
                1 => {
                    std::env::set_current_dir(&matches[0])?;
                    println!("切换到: {}", matches[0]);
                }
                _ => {
                    if let Some(selected) = History::select_fzf(&matches) {
                        std::env::set_current_dir(&selected)?;
                        println!("切换到: {selected}");
                    } else {
                        anyhow::bail!("未选择目录");
                    }
                }
            }
        }
        None => {
            let cwd = std::env::current_dir()?;
            let mut history = History::load();
            history.add(&cwd);
            history.save();
        }
    }
    Ok(())
}

/// 构建配置、AI、搜索与管线并执行完整提交流程
fn run_flow(o: FlowOptions) {
    if let Err(e) = run_flow_inner(o) {
        eprintln!("{}", color::red(&e.to_string()));
        std::process::exit(1);
    }
}

fn run_flow_inner(o: FlowOptions) -> anyhow::Result<()> {
    let skill_path = o.skill_path.unwrap_or_default();
    let lazygit_path = o.lazygit_config.unwrap_or_default();
    let rg_pattern = o.rg_pattern.unwrap_or_default();
    let fd_pattern = o.fd_pattern.unwrap_or_default();

    let app_config = AppConfig::load(&lazygit_path, &skill_path, o.config_dir, o.api_key)?;
    println!(
        "使用 LLM: {} ({})",
        color::cyan(&app_config.ai.model),
        host_of(&app_config.ai.base_url)
    );

    let git = Arc::new(RealGit) as Arc<dyn GitBackend>;
    let ai = agc::ai::ProviderRegistry::new().create(&app_config.ai.provider, app_config.ai.clone())?;
    let search = Arc::new(RealSearch) as Arc<dyn agc::search::SearchBackend>;

    let mut builder = PipelineBuilder::new()
        .with_git(git)
        .with_ai(ai)
        .with_search(search)
        .with_config(agc::config::Config {
            types: app_config.types,
            scopes: app_config.scopes,
        })
        .with_rg_pattern(rg_pattern)
        .with_fd_pattern(fd_pattern);
    if let Some((remote, branch)) = &o.auto_push {
        builder = builder.with_auto_push(remote.clone(), branch.clone());
    }

    let pipeline = builder.build()?;

    let mut ctx = PipelineContext::new(o.auto_push, o.auto_commit);
    pipeline.run(&mut ctx)?;
    Ok(())
}

/// 从 base_url 提取域名（去掉协议与路径）
fn host_of(url: &str) -> &str {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    rest.split('/').next().unwrap_or(rest)
}
