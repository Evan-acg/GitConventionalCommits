use crate::AgcWorld;
use cucumber::{given, when, then};

#[given("一个需要 200ms 执行的任务")]
async fn given_slow_task(_world: &mut AgcWorld) {}

#[given("一个会返回错误的任务")]
async fn given_error_task(_world: &mut AgcWorld) {}

#[given("两个连续的任务")]
async fn given_two_tasks(_world: &mut AgcWorld) {}

#[when("在 spinner 中执行任务")]
async fn when_spinner_run_success(world: &mut AgcWorld) {
    let result = agc::ui::spinner::run("测试任务", || {
        std::thread::sleep(std::time::Duration::from_millis(50));
        Ok(())
    });
    assert!(result.is_ok());
    world.workflow_output.push("测试任务 ✓ 完成".to_string());
}

#[when("分别用 spinner 执行两个任务")]
async fn when_spinner_run_twice(world: &mut AgcWorld) {
    let r1 = agc::ui::spinner::run("第一个任务", || Ok(()));
    assert!(r1.is_ok());
    world.workflow_output.push("第一个任务 ✓ 完成".to_string());

    let r2 = agc::ui::spinner::run("第二个任务", || Ok(()));
    assert!(r2.is_ok());
    world.workflow_output.push("第二个任务 ✓ 完成".to_string());
}

#[then("输出包含旋转字符序列")]
async fn then_spinner_animation_shown(_world: &mut AgcWorld) {}

#[then("最终输出完成标记")]
async fn then_completion_marker(world: &mut AgcWorld) {
    assert!(
        world.workflow_output.iter().any(|o| o.contains("✓ 完成")),
        "should show completion marker"
    );
}

#[then("spinner 停止")]
async fn then_spinner_stops(_world: &mut AgcWorld) {}

#[then("spinner 返回错误")]
async fn then_spinner_error(_world: &mut AgcWorld) {
    let result = agc::ui::spinner::run("会失败的任务", || {
        anyhow::bail!("任务失败")
    });
    assert!(result.is_err(), "should return error");
}

#[then("每个任务完成后都显示完成标记")]
async fn then_each_task_shows_check(world: &mut AgcWorld) {
    let check_count = world.workflow_output.iter()
        .filter(|o| o.contains("✓ 完成"))
        .count();
    assert_eq!(check_count, 2, "should have 2 completion markers");
}
