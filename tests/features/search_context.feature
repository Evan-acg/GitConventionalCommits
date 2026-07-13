Feature: 搜索上下文
  作为 agc 工具
  我需要通过 rg 和 fd 搜索变更文件的上下文
  以便 AI 能生成更精准的 commit 消息

  Scenario: rg 搜索函数/结构体定义
    Given 变更文件列表包含指定文件
    And rg 已安装
    When 执行 rg 搜索默认模式
    Then 返回文件中匹配函数定义的行

  Scenario: rg 不可用时静默提示
    Given rg 未安装
    When 执行 rg 搜索
    Then 输出建议安装 rg 的提示
    And rg 返回空字符串

  Scenario: rg 自定义模式搜索
    Given 指定 rg 模式为 pub fn
    When 执行 rg 搜索
    Then 使用自定义模式而非默认模式

  Scenario: rg 变更文件列表为空
    Given 变更文件列表为空
    When 执行 rg 搜索
    Then rg 返回空字符串

  Scenario: fd 查找同名不同后缀文件
    Given 变更文件包含 src/main.rs
    When 执行 fd 搜索
    Then 查找同名的关联文件

  Scenario: fd 列出目录内文件
    Given 文件在 src/ 目录下
    When 执行 fd 目录搜索
    Then 返回目录所有文件列表

  Scenario: fd 自定义模式搜索
    Given 指定 fd 模式为 *.toml
    When 执行 fd 搜索
    Then 返回自定义模式搜索结果

  Scenario: 合并 rg 和 fd 上下文
    Given rg 返回 fn main()
    And fd 返回 src/lib.rs
    When 合并上下文
    Then 结果包含两部分内容并以分隔线分割
