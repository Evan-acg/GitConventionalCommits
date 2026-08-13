Feature: 配置初始化
  作为 agc 工具
  我需要初始化 AI 配置文件
  以便快速搭建 provider 配置

  Scenario: 在指定目录生成配置文件
    Given 指定 init 目标目录
    When 执行 init 生成配置
    Then 生成 default.yaml 且包含 provider 字段

  Scenario: 已存在配置文件时拒绝覆盖
    Given 指定 init 目标目录
    And 该目录已存在 default.yaml
    When 执行 init 生成配置
    Then 报错提示文件已存在

  Scenario: 使用 --force 覆盖已有配置
    Given 指定 init 目标目录
    And 该目录已存在 default.yaml
    When 使用 --force 执行 init
    Then 生成 default.yaml 且包含 provider 字段
