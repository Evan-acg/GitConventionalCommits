use std::sync::Arc;

use agc::ai::openai::OpenAI;
use agc::cli::Cli;
use agc::config::ConfigChain;
use agc::git::real::RealGit;
use agc::history::History;
use agc::pipeline::{PipelineBuilder, PipelineContext};
use agc::search::RealSearch;
use agc::ui::color;

fn main() {
    let cli = Cli::parse();

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

    // ── 步骤2：构建配置 ──
    let skill_path = cli.skill_path.unwrap_or_default();
    let lazygit_path = cli.lazygit_config.unwrap_or_default();
    let rg_pattern = cli.rg_pattern.unwrap_or_default();
    let fd_pattern = cli.fd_pattern.unwrap_or_default();

    let api_key = if !cli.api_key.as_deref().unwrap_or("").is_empty() {
        cli.api_key.unwrap()
    } else {
        match std::env::var("MESSAGE_API_KEY") {
            Ok(key) => key,
            Err(_) => {
                eprintln!("MESSAGE_API_KEY 未设置，可通过 --api-key 参数或 MESSAGE_API_KEY 环境变量设置");
                std::process::exit(1);
            }
        }
    };
    let model = std::env::var("OPENAI_MODEL")
        .unwrap_or_else(|_| "deepseek-v4-flash".to_string());
    let base_url = std::env::var("OPENAI_BASE_URL")
        .unwrap_or_else(|_| "https://api.deepseek.com".to_string());

    let config = ConfigChain::new(&lazygit_path, &skill_path).load();
    let git = Arc::new(RealGit) as Arc<dyn agc::git::GitBackend>;
    let ai = Arc::new(OpenAI::new(api_key, model, base_url)) as Arc<dyn agc::ai::AiProvider>;
    let search = Arc::new(RealSearch) as Arc<dyn agc::search::SearchBackend>;

    // ── 步骤3：构建并执行管线 ──
    let pipeline = match PipelineBuilder::new()
        .with_git(git)
        .with_ai(ai)
        .with_search(search)
        .with_config(config)
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

    let mut ctx = PipelineContext::new(cli.push);
    if let Err(e) = pipeline.run(&mut ctx) {
        eprintln!("{}", color::red(&e.to_string()));
        std::process::exit(1);
    }
}
