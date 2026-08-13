use std::sync::Arc;

use agc::cli::{Cli, SubCommands};
use agc::config::AppConfig;
use agc::git::real::RealGit;
use agc::git::GitBackend;
use agc::history::History;
use agc::pipeline::{PipelineBuilder, PipelineContext};
use agc::search::RealSearch;
use agc::ui::color;

fn main() {
    let cli = Cli::parse();

    // ── 步骤0：init 子命令（初始化配置文件） ──
    if let Some(SubCommands::Init(args)) = &cli.sub {
        match agc::config::init::run(args.path.clone(), args.force) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("{}", color::red(&e.to_string()));
                std::process::exit(1);
            }
        }
        return;
    }

    // ── 步骤1：目录选择 ──
    match &cli.directory {
        Some(pattern) => {
            let history = History::load();
            let matches = history.filter(pattern);
            match matches.len() {
                0 => {
                    eprintln!("没有匹配的历史目录: {pattern}");
                    std::process::exit(1);
                }
                1 => {
                    if let Err(e) = std::env::set_current_dir(&matches[0]) {
                        eprintln!("无法切换到目录: {e}");
                        std::process::exit(1);
                    }
                    println!("切换到: {}", matches[0]);
                }
                _ => {
                    if let Some(selected) = History::select_fzf(&matches) {
                        if let Err(e) = std::env::set_current_dir(&selected) {
                            eprintln!("无法切换到目录: {e}");
                            std::process::exit(1);
                        }
                        println!("切换到: {selected}");
                    } else {
                        std::process::exit(1);
                    }
                }
            }
        }
        None => {
            let cwd = match std::env::current_dir() {
                Ok(cwd) => cwd,
                Err(e) => {
                    eprintln!("无法获取当前目录: {e}");
                    std::process::exit(1);
                }
            };
            let mut history = History::load();
            history.add(&cwd);
            history.save();
        }
    }

    // ── 步骤1.5：--pull 短路（仅拉取，不进行 AI 提交） ──
    if let Some(remote) = cli.pull {
        let git = RealGit;
        match git.pull(&remote) {
            Ok(()) => println!("{}", color::green(&format!("git pull {remote} 成功"))),
            Err(e) => {
                eprintln!("{}", color::red(&e.to_string()));
                std::process::exit(1);
            }
        }
        return;
    }

    // ── 步骤2：构建配置 ──
    let skill_path = cli.skill_path.unwrap_or_default();
    let lazygit_path = cli.lazygit_config.unwrap_or_default();
    let rg_pattern = cli.rg_pattern.unwrap_or_default();
    let fd_pattern = cli.fd_pattern.unwrap_or_default();

    let app_config = match AppConfig::load(
        &lazygit_path,
        &skill_path,
        cli.config_dir.clone(),
        cli.api_key.clone(),
    ) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("{}", color::red(&e.to_string()));
            std::process::exit(1);
        }
    };

    let git = Arc::new(RealGit) as Arc<dyn agc::git::GitBackend>;
    let ai = match agc::ai::ProviderRegistry::new()
        .create(&app_config.ai.provider, app_config.ai.clone())
    {
        Ok(provider) => provider,
        Err(e) => {
            eprintln!("{}", color::red(&e.to_string()));
            std::process::exit(1);
        }
    };
    let search = Arc::new(RealSearch) as Arc<dyn agc::search::SearchBackend>;

    // ── 步骤3：构建并执行管线 ──
    let pipeline = match PipelineBuilder::new()
        .with_git(git)
        .with_ai(ai)
        .with_search(search)
        .with_config(agc::config::Config {
            types: app_config.types,
            scopes: app_config.scopes,
        })
        .with_rg_pattern(rg_pattern)
        .with_fd_pattern(fd_pattern)
        .with_auto_push(cli.push.clone())
        .build()
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", color::red(&e.to_string()));
            std::process::exit(1);
        }
    };

    let mut ctx = PipelineContext::new(cli.push, cli.yes);
    if let Err(e) = pipeline.run(&mut ctx) {
        eprintln!("{}", color::red(&e.to_string()));
        std::process::exit(1);
    }
}
