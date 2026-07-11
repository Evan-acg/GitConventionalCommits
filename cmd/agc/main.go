package main

import (
	"flag"
	"fmt"
	"os"

	"github.com/Evan-acg/GitConventionalCommits/internal/ai"
	"github.com/Evan-acg/GitConventionalCommits/internal/commit"
	"github.com/Evan-acg/GitConventionalCommits/internal/config"
	"github.com/Evan-acg/GitConventionalCommits/internal/git"
)

func main() {
	skillPath := flag.String("skill-path", "", "path to git-commit SKILL.md")
	apiKey := flag.String("api-key", "", "API key（优先级高于 MESSAGE_API_KEY 环境变量）")
	flag.Parse()

	if v := os.Getenv("GIT_COMMIT_SKILL_PATH"); v != "" && *skillPath == "" {
		*skillPath = v
	}

	typeList, scopeList := config.Load(*skillPath)

	diff, err := git.Diff()
	if err != nil {
		fmt.Fprintf(os.Stderr, "获取 diff 失败: %v\n", err)
		os.Exit(1)
	}
	if diff == "" {
		fmt.Println("没有未暂存的变更")
		return
	}

	msg, err := ai.Generate(typeList, scopeList, diff, *apiKey)
	if err != nil {
		fmt.Fprintf(os.Stderr, "AI 生成消息失败: %v\n", err)
		os.Exit(1)
	}

	msg = commit.Capitalize(msg)

	if commit.Confirm(msg) {
		if err := commit.Execute(msg); err != nil {
			fmt.Fprintf(os.Stderr, "提交失败: %v\n", err)
			os.Exit(1)
		}
		fmt.Println("提交成功")
	} else {
		fmt.Println("已取消")
	}
}
