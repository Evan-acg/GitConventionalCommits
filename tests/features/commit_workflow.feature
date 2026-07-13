Feature: 提交工作流
  作为用户
  我需要通过 agc 完成从分析到提交的完整流程
  以便高效生成规范化的 commit 消息

  Scenario: 工作区无变更时提前退出
    Given 工作区没有未暂存的变更
    And 没有未跟踪的文件
    When 执行完整工作流
    Then 输出 没有未暂存的变更
    And 不调用 AI

  Scenario: 用户取消提交
    Given AI 已返回有效条目
    And 用户输入非 no
    When 执行确认步骤
    Then 输出 已取消
    And 不执行 git commit

  Scenario: AI 返回多条条目时循环处理
    Given AI 返回 2 条独立 commit 条目
    And 用户每次输入 ok
    When 执行完整工作流
    Then git commit 被执行 2 次

  Scenario: AI 返回无效条目被过滤
    Given AI 返回的条目中有一条缺少 type
    And 还有一条有效条目
    And 用户输入 ok
    When 执行完整工作流
    Then 输出警告跳过无效条目
    And git commit 被执行 1 次

  Scenario: 选择性 Stage 文件
    Given AI 返回的条目包含 files 字段
    And 用户输入 ok
    When 执行完整工作流
    Then 只 stage 指定的文件
