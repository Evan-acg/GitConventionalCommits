Feature: 配置加载
  作为 agc 工具
  我需要加载 type 和 scope 配置
  以便 AI 能生成符合项目规范的 commit 消息

  Scenario: 从 .lazygit.yaml 加载有效配置
    Given 存在 .lazygit.yaml 文件包含 type 和 scope 定义
    When 加载配置
    Then 返回的 types 列表包含 Feat
    And 返回的 scopes 列表非空

  Scenario: YAML 不存在时回退到 SKILL.md
    Given .lazygit.yaml 不存在
    And 存在 SKILL.md 包含 markdown 表格
    When 加载配置
    Then 返回的 types 列表包含 Feat

  Scenario: YAML 和 SKILL.md 都不存在时使用默认值
    Given .lazygit.yaml 不存在
    And SKILL.md 不存在
    When 加载配置
    Then 返回默认 types 列表包含 Feat 和 Fix

  Scenario: YAML 中 types 为空时回退到 SKILL.md
    Given 存在 .lazygit.yaml 文件 types 为空数组
    When 加载配置
    Then 触发回退到 SKILL.md 解析

  Scenario: SKILL.md 中提取 markdown 表格格式 type
    Given SKILL.md 包含 markdown type 行
    When 加载配置
    Then type 列表包含 Refactor 首字母大写

  Scenario: 自定义 lazygit 路径
    Given 存在自定义配置路径
    When 使用自定义路径加载配置
    Then 正确从该路径加载 type 和 scope
