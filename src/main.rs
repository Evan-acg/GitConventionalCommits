use agc::cli::Cli;
use agc::workflow;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let skill_path = cli.skill_path.unwrap_or_default();
    let api_key = cli.api_key.unwrap_or_default();
    let rg_pattern = cli.rg_pattern.unwrap_or_default();
    let fd_pattern = cli.fd_pattern.unwrap_or_default();
    let lazygit_path = cli.lazygit_config.unwrap_or_default();

    let wf = workflow::build_default_workflow(
        &skill_path,
        &lazygit_path,
        &api_key,
        &rg_pattern,
        &fd_pattern,
    )?;

    wf.run()
}
