use crate::AgcWorld;
use cucumber::{given, when, then};
use agc::config::ConfigChain;
use std::fs;

#[given("存在 .lazygit.yaml 文件包含 type 和 scope 定义")]
async fn given_lazygit_yaml_exists(world: &mut AgcWorld) {
    let dir = tempfile::tempdir().unwrap();
    let yaml_path = dir.path().join(".lazygit.yaml");
    let content = "type:\n  - name: Feat\n    docs: 新特性\n  - name: Fix\n    docs: 修补校验\n  - name: Docs\n    docs: 文档变更\nscopes:\n  - name: Scripts\n    docs: 脚本文件相关\n  - name: Asserts\n    docs: 资源文件\n";
    fs::write(&yaml_path, content).unwrap();
    world.config_lazygit_path = yaml_path.to_string_lossy().to_string();
    world.temp_dir = Some(dir);
}

#[given(".lazygit.yaml 不存在")]
async fn given_lazygit_yaml_not_exists(world: &mut AgcWorld) {
    world.config_lazygit_path = "nonexistent.yaml".to_string();
}

#[given(regex = r"存在 SKILL\.md 包含 markdown 表格")]
async fn given_skill_md_exists(world: &mut AgcWorld) {
    let dir = tempfile::tempdir().unwrap();
    let skill_path = dir.path().join("SKILL.md");
    let content = "| `feat` | 新特性 |\n";
    fs::write(&skill_path, content).unwrap();
    world.config_skill_path = skill_path.to_string_lossy().to_string();
    world.temp_dir = Some(dir);
}

#[given("SKILL.md 不存在")]
async fn given_skill_md_not_exists(world: &mut AgcWorld) {
    world.config_skill_path = "nonexistent_skill.md".to_string();
}

#[given("存在 .lazygit.yaml 文件 types 为空数组")]
async fn given_lazygit_yaml_empty_types(world: &mut AgcWorld) {
    let dir = tempfile::tempdir().unwrap();
    let yaml_path = dir.path().join(".lazygit.yaml");
    let content = "type: []\nscopes:\n";
    fs::write(&yaml_path, content).unwrap();
    world.config_lazygit_path = yaml_path.to_string_lossy().to_string();
    let skill_path = dir.path().join("SKILL.md");
    fs::write(&skill_path, "| `feat` | 新特性 |\n").unwrap();
    world.config_skill_path = skill_path.to_string_lossy().to_string();
    world.temp_dir = Some(dir);
}

#[given(regex = r"SKILL\.md 包含 markdown type 行")]
async fn given_skill_md_refactor(world: &mut AgcWorld) {
    let dir = tempfile::tempdir().unwrap();
    let skill_path = dir.path().join("SKILL.md");
    let content = "| `refactor` | 重构 |\n";
    fs::write(&skill_path, content).unwrap();
    world.config_skill_path = skill_path.to_string_lossy().to_string();
    world.temp_dir = Some(dir);
}

#[given(regex = r"存在自定义配置路径")]
async fn given_custom_yaml_exists(world: &mut AgcWorld) {
    let dir = tempfile::tempdir().unwrap();
    let custom_dir = dir.path().join("custom").join("path");
    fs::create_dir_all(&custom_dir).unwrap();
    let yaml_path = custom_dir.join("config.yaml");
    let content = "type:\n  - name: CustomType\nscopes:\n  - name: CustomScope\n";
    fs::write(&yaml_path, content).unwrap();
    world.config_lazygit_path = yaml_path.to_string_lossy().to_string();
    world.temp_dir = Some(dir);
}

#[when("加载配置")]
async fn when_load_config(world: &mut AgcWorld) {
    let cur_dir = std::env::current_dir().ok();
    if let Some(ref dir) = world.temp_dir {
        std::env::set_current_dir(dir.path()).ok();
    }
    let cfg = ConfigChain::new(&world.config_lazygit_path, &world.config_skill_path).load();
    world.loaded_types = cfg.types;
    world.loaded_scopes = cfg.scopes;
    if let Some(dir) = cur_dir {
        std::env::set_current_dir(dir).ok();
    }
}

#[when("使用自定义路径加载配置")]
async fn when_load_custom_config(world: &mut AgcWorld) {
    let cfg = ConfigChain::new(&world.config_lazygit_path, "").load();
    world.loaded_types = cfg.types;
    world.loaded_scopes = cfg.scopes;
}

#[then("返回的 types 列表包含 Feat")]
async fn then_types_contain_feat(world: &mut AgcWorld) {
    assert!(world.loaded_types.contains(&"Feat".to_string()), "types should contain Feat");
}

#[then("返回的 scopes 列表非空")]
async fn then_scopes_nonempty(world: &mut AgcWorld) {
    assert!(!world.loaded_scopes.is_empty(), "scopes should not be empty");
}

#[then(regex = r"^返回默认 types 列表包含 (.*) 和 (.*)$")]
async fn then_default_types_contain(world: &mut AgcWorld, t1: String, t2: String) {
    assert!(world.loaded_types.contains(&t1), "should contain {t1}");
    assert!(world.loaded_types.contains(&t2), "should contain {t2}");
}

#[then("触发回退到 SKILL.md 解析")]
async fn then_fallback_to_skill(world: &mut AgcWorld) {
    assert!(world.loaded_types.contains(&"Feat".to_string()), "should fallback to SKILL.md");
}

#[then(regex = r"^type 列表包含 (.*) 首字母大写$")]
async fn then_type_capitalized(world: &mut AgcWorld, expected: String) {
    assert!(
        world.loaded_types.contains(&expected),
        "type should be capitalized: {expected}, got {:?}",
        world.loaded_types
    );
}

#[then("正确从该路径加载 type 和 scope")]
async fn then_custom_path_loaded(world: &mut AgcWorld) {
    assert!(world.loaded_types.contains(&"CustomType".to_string()));
    assert!(world.loaded_scopes.contains(&"CustomScope".to_string()));
}
