use crate::AgcWorld;
use cucumber::{given, when, then};
use std::fs;

#[given("指定 init 目标目录")]
async fn given_init_dir(world: &mut AgcWorld) {
    let dir = tempfile::tempdir().unwrap();
    world.init_path = dir.path().to_string_lossy().to_string();
    world.init_dir = Some(dir);
}

#[given("该目录已存在 default.yaml")]
async fn given_init_file_exists(world: &mut AgcWorld) {
    let dir = world.init_dir.as_ref().unwrap();
    fs::write(dir.path().join("default.yaml"), "existing").unwrap();
}

#[when("执行 init 生成配置")]
async fn when_init(world: &mut AgcWorld) {
    world.init_error = match agc::config::init::run(Some(world.init_path.clone()), false) {
        Ok(()) => None,
        Err(e) => Some(e.to_string()),
    };
}

#[when("使用 --force 执行 init")]
async fn when_init_force(world: &mut AgcWorld) {
    world.init_error = match agc::config::init::run(Some(world.init_path.clone()), true) {
        Ok(()) => None,
        Err(e) => Some(e.to_string()),
    };
}

#[then("生成 default.yaml 且包含 provider 字段")]
async fn then_init_file_created(world: &mut AgcWorld) {
    assert!(
        world.init_error.is_none(),
        "init should succeed, got: {:?}",
        world.init_error
    );
    let path = world.init_dir.as_ref().unwrap().path().join("default.yaml");
    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("use_provider:") && content.contains("providers:"),
        "template should contain use_provider and providers fields"
    );
}

#[then("报错提示文件已存在")]
async fn then_init_exists_error(world: &mut AgcWorld) {
    let err = world.init_error.as_deref().expect("init should error");
    assert!(err.contains("已存在"), "error should mention exists: {err}");
}
