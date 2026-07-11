package main

import (
	"flag"
	"fmt"
	"os"

	"github.com/Evan-acg/GitConventionalCommits/internal/ai"
	"github.com/Evan-acg/GitConventionalCommits/internal/commit"
	"github.com/Evan-acg/GitConventionalCommits/internal/config"
	"github.com/Evan-acg/GitConventionalCommits/internal/git"
	"github.com/Evan-acg/GitConventionalCommits/internal/search"
)

func main() {
	skillPath := flag.String("skill-path", "", "path to git-commit SKILL.md")
	apiKey := flag.String("api-key", "", "API key（优先级高于 MESSAGE_API_KEY 环境变量）")
	rgPattern := flag.String("rg-pattern", "", "rg 搜索模式，不指定则自动检测代码结构")
	fdPattern := flag.String("fd-pattern", "", "fd 搜索模式，查找关联文件")
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

	changedFiles, _ := git.ChangedFiles()
	rgContext := search.Context(changedFiles, *rgPattern)
	fdContext := search.RelatedFiles(changedFiles, *fdPattern)

	var extraContext string
	if rgContext != "" {
		extraContext += "---\n变更文件结构上下文:\n" + rgContext
	}
	if fdContext != "" {
		if extraContext != "" {
			extraContext += "\n"
		}
		extraContext += "---\n关联文件上下文:\n" + fdContext
	}

	msg, err := ai.Generate(typeList, scopeList, diff, *apiKey, extraContext)
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
