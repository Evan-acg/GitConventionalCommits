package app

import (
	"context"
	"fmt"
	"os"
	"strings"

	"github.com/Evan-acg/GitConventionalCommits/internal/ai"
	"github.com/Evan-acg/GitConventionalCommits/internal/commit"
	"github.com/Evan-acg/GitConventionalCommits/internal/config"
	"github.com/Evan-acg/GitConventionalCommits/internal/git"
	"github.com/Evan-acg/GitConventionalCommits/internal/search"
	"github.com/Evan-acg/GitConventionalCommits/internal/strutil"
)

type Options struct {
	SkillPath   string
	APIKey      string
	RGPattern   string
	FDPattern   string
	LazyGitPath string
}

type Workflow struct{}

func (Workflow) Run(ctx context.Context, opts Options) error {
	typeList, scopeList := config.Load(opts.SkillPath, opts.LazyGitPath)

	diff, err := git.Diff()
	if err != nil {
		return fmt.Errorf("获取 diff 失败: %w", err)
	}
	if diff == "" {
		fmt.Println("没有未暂存的变更")
		return nil
	}

	changedFiles, err := git.ChangedFiles()
	if err != nil {
		fmt.Fprintf(os.Stderr, "警告: 获取变更文件列表失败: %v\n", err)
	}

	rgContext := search.RGContext(changedFiles, opts.RGPattern)
	fdContext := search.FDContext(changedFiles, opts.FDPattern)
	extraContext := mergeContext(rgContext, fdContext)

	model := os.Getenv("OPENAI_MODEL")
	if model == "" {
		model = "gpt-4o-mini"
	}

	apiKey := opts.APIKey
	if apiKey == "" {
		apiKey = os.Getenv("MESSAGE_API_KEY")
	}
	if apiKey == "" {
		return fmt.Errorf("MESSAGE_API_KEY 未设置，可通过 --api-key 参数或 MESSAGE_API_KEY 环境变量设置")
	}

	llm := &ai.OpenAI{
		APIKey:  apiKey,
		Model:   model,
		BaseURL: "https://api.openai.com",
	}

	msg, err := llm.Generate(ctx, ai.Request{
		Types:        typeList,
		Scopes:       scopeList,
		Diff:         diff,
		ExtraContext: extraContext,
	})
	if err != nil {
		return fmt.Errorf("AI 生成消息失败: %w", err)
	}

	msg = strutil.CapitalizeMessage(msg)

	if !commit.Confirm(msg) {
		fmt.Println("已取消")
		return nil
	}

	if err := commit.Stage(); err != nil {
		return fmt.Errorf("git add 失败: %w", err)
	}
	if err := commit.Commit(msg); err != nil {
		return fmt.Errorf("git commit 失败: %w", err)
	}
	fmt.Println("提交成功")
	return nil
}

func mergeContext(rg, fd string) string {
	var sb strings.Builder
	if rg != "" {
		sb.WriteString("---\n变更文件结构上下文:\n" + rg)
	}
	if fd != "" {
		if sb.Len() > 0 {
			sb.WriteString("\n")
		}
		sb.WriteString("---\n关联文件上下文:\n" + fd)
	}
	return sb.String()
}
