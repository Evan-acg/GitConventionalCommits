Feature: 提交消息格式化
  作为 agc 工具
  我需要按 conventional commit 格式输出消息
  以便可以直接用于 git commit

  Scenario: 格式化基本提交消息
    Given 单个 Entry 包含 type scope message
    When 格式化消息
    Then 格式化结果为 Feat(Git): 添加 diff 函数

  Scenario: 格式化带详细描述的提交消息
    Given Entry 带详细描述
    When 格式化消息
    Then detail 包含在格式化结果中

  Scenario: 首字母大写处理
    Given 原始字符串 feat
    When 应用 Capitalize 处理
    Then 字符串结果为 Feat

  Scenario: 空字符串的 Capitalize
    Given 输入的字符串为空
    When 应用 Capitalize 处理
    Then 字符串结果为空

  Scenario: 过滤缺少必填字段的 Entry
    Given 存在 entries 其中一个缺少 message 字段
    When 验证条目有效性
    Then 该条目被过滤并输出警告
