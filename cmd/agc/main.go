package main

import (
	"context"
	"flag"
	"fmt"
	"os"

	"github.com/Evan-acg/GitConventionalCommits/internal/app"
)

func main() {
	skillPath := flag.String("skill-path", "", "path to git-commit SKILL.md")
	apiKey := flag.String("api-key", "", "API key（优先级高于 MESSAGE_API_KEY 环境变量）")
	rgPattern := flag.String("rg-pattern", "", "rg 搜索模式，不指定则自动检测代码结构")
	fdPattern := flag.String("fd-pattern", "", "fd 搜索模式，查找关联文件")
	lazygitPath := flag.String("lazygit-config", "", "lazygit 配置路径（默认 .lazygit.yaml）")
	flag.Parse()

	if v := os.Getenv("GIT_COMMIT_SKILL_PATH"); v != "" && *skillPath == "" {
		*skillPath = v
	}

	var w app.Workflow
	if err := w.Run(context.Background(), app.Options{
		SkillPath:   *skillPath,
		APIKey:      *apiKey,
		RGPattern:   *rgPattern,
		FDPattern:   *fdPattern,
		LazyGitPath: *lazygitPath,
	}); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
