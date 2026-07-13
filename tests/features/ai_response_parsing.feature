Feature: AI 响应解析
  作为 agc 工具
  我需要解析 AI 返回的 JSON 格式响应
  以便提取 commit 条目

  Scenario: 解析单条提交的 JSON 响应
    Given AI 返回单条 JSON 响应
    When 解析响应
    Then 返回 1 条 Entry
    And Entry type 为 Feat
    And Entry scope 为 Git

  Scenario: 解析多条提交的 JSON 响应
    Given AI 返回包含 2 条 data 的 JSON
    When 解析响应
    Then 返回 2 条 Entry

  Scenario: 解析包含 reason 的响应
    Given AI 返回带 reason 的 JSON
    When 解析响应
    Then 同时返回 reason 字符串

  Scenario: 解析包含 files 的完整条目
    Given AI 返回带 files 的 JSON
    When 解析响应
    Then Entry files 包含指定的文件路径

  Scenario: 解析包含 detail 的条目
    Given AI 返回带 detail 的 JSON
    When 解析响应
    Then Entry detail 包含详细描述

  Scenario: 处理无效 JSON 响应
    Given AI 返回非 JSON 字符串
    When 解析响应
    Then 返回空 entries 列表

  Scenario: 处理空 JSON 响应
    Given AI 返回空 JSON 对象
    When 解析响应
    Then 返回空 entries 列表
