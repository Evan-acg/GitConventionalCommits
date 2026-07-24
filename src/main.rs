use agc::cli::Cli;
use agc::ui::color;
use agc::workflow;

fn main() {
    let cli = Cli::parse();

    let skill_path = cli.skill_path.unwrap_or_default();
    let api_key = cli.api_key.unwrap_or_default();
    let rg_pattern = cli.rg_pattern.unwrap_or_default();
    let fd_pattern = cli.fd_pattern.unwrap_or_default();
    let lazygit_path = cli.lazygit_config.unwrap_or_default();

    match workflow::build_default_workflow(
        &skill_path,
        &lazygit_path,
        &api_key,
        &rg_pattern,
        &fd_pattern,
    ) {
        Ok(wf) => {
            if let Err(e) = wf.run() {
                eprintln!("{}", color::red(&e.to_string()));
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("{}", color::red(&e.to_string()));
            std::process::exit(1);
        }
    }
}
