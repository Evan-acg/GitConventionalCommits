Feature: Git 信息收集
  作为 agc 工具
  我需要获取工作区的 git 变更信息
  以便提供给 AI 生成准确的 commit 消息

  Scenario: 获取工作区 diff
    Given 工作区有未暂存的变更
    When 执行 git diff
    Then 返回包含变更内容的 diff 字符串

  Scenario: 工作区无变更时返回空
    Given 工作区没有未暂存的变更
    When 执行 git diff
    Then 返回空字符串的 git 信息

  Scenario: 获取已变更文件列表
    Given 工作区有文件修改
    When 获取变更文件列表
    Then 返回只包含已修改文件路径的列表

  Scenario: 获取工作区状态
    Given 工作区有 staged 变更
    When 获取 status short
    Then 返回 git status short 格式的输出

  Scenario: 获取 diff 统计信息
    Given 工作区有未暂存的变更
    When 获取 diff stat
    Then 返回包含插入删除行数的统计信息

  Scenario: 获取已暂存变更统计
    Given 工作区有 staged 变更
    When 获取 diff cached stat
    Then 返回已暂存变更的统计信息

  Scenario: 获取未跟踪文件
    Given 工作区有未跟踪的文件
    When 获取未跟踪文件列表
    Then 返回未跟踪文件的路径列表

  Scenario: 读取未跟踪文件内容
    Given 存在未跟踪的文本文件小于 100KB
    When 读取未跟踪文件内容
    Then 返回包含文件内容的字符串

  Scenario: 跳过超过 100KB 的未跟踪文件
    Given 存在未跟踪的文件超过 100KB
    When 读取未跟踪文件内容
    Then 跳过该文件并给出跳过提示

  Scenario: 跳过二进制未跟踪文件
    Given 存在未跟踪的二进制文件
    When 读取未跟踪文件内容
    Then 跳过该文件并给出二进制提示

  Scenario: git 命令失败时返回错误
    Given 不在 git 仓库中
    When 执行 git diff
    Then 返回错误的 diff
