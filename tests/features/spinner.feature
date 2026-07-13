Feature: Spinner 动画
  作为用户
  我需要看到任务执行中的旋转动画
  以便知道程序正在运行

  Scenario: 显示 spinner 并在成功后输出完成信息
    Given 一个需要 200ms 执行的任务
    When 在 spinner 中执行任务
    Then 输出包含旋转字符序列
    And 最终输出完成标记

  Scenario: spinner 中任务失败
    Given 一个会返回错误的任务
    When 在 spinner 中执行任务
    Then spinner 停止
    And spinner 返回错误

  Scenario: 多层 spinner（嵌套）
    Given 两个连续的任务
    When 分别用 spinner 执行两个任务
    Then 每个任务完成后都显示完成标记
