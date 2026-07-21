package app

import (
	"context"
	"fmt"
	"os"
	"strings"

	"github.com/Evan-acg/GitConventionalCommits/internal/ai"
	"github.com/Evan-acg/GitConventionalCommits/internal/color"
	"github.com/Evan-acg/GitConventionalCommits/internal/commit"
	"github.com/Evan-acg/GitConventionalCommits/internal/config"
	"github.com/Evan-acg/GitConventionalCommits/internal/git"
	"github.com/Evan-acg/GitConventionalCommits/internal/search"
	"github.com/Evan-acg/GitConventionalCommits/internal/spinner"
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

	model := os.Getenv("OPENAI_MODEL")
	if model == "" {
		model = "deepseek-v4-flash"
	}

	apiKey := opts.APIKey
	if apiKey == "" {
		apiKey = os.Getenv("MESSAGE_API_KEY")
	}
	if apiKey == "" {
		return fmt.Errorf("MESSAGE_API_KEY 未设置，可通过 --api-key 参数或 MESSAGE_API_KEY 环境变量设置")
	}

	baseURL := os.Getenv("OPENAI_BASE_URL")
	if baseURL == "" {
		baseURL = "https://api.deepseek.com"
	}

	llm := &ai.OpenAI{
		APIKey:  apiKey,
		Model:   model,
		BaseURL: baseURL,
	}

	hasStaged, err := hasStagedChanges()
	if err != nil {
		return err
	}

	if hasStaged {
		if err := handleStagedChanges(ctx, llm, typeList, scopeList, opts); err != nil {
			return err
		}
	}

	return handleUnstagedChanges(ctx, llm, typeList, scopeList, opts)
}

func hasStagedChanges() (bool, error) {
	files, err := git.StagedFiles()
	if err != nil {
		return false, fmt.Errorf("获取已暂存文件列表失败: %w", err)
	}
	return len(files) > 0, nil
}

func handleStagedChanges(ctx context.Context, llm *ai.OpenAI, typeList, scopeList []string, opts Options) error {
	var (
		stagedDiff  string
		stagedFiles []string
		statusShort string
	)

	if err := spinner.Run("正在获取已暂存 git 变更信息", func() error {
		d, err := git.DiffCached()
		if err != nil {
			return err
		}
		stagedDiff = d

		sf, err := git.StagedFiles()
		if err != nil {
			return fmt.Errorf("获取已暂存文件列表失败: %w", err)
		}
		stagedFiles = sf

		statusShort, _ = git.StatusShort()
		return nil
	}); err != nil {
		return fmt.Errorf("获取已暂存 git 信息失败: %w", err)
	}

	if stagedDiff == "" {
		return nil
	}

	var gitInfo strings.Builder
	if statusShort != "" {
		gitInfo.WriteString("--- 工作区状态 ---\n" + statusShort + "\n")
	}
	diffCachedStat, _ := git.DiffCachedStat()
	if diffCachedStat != "" {
		gitInfo.WriteString("--- 已暂存变更摘要 ---\n" + diffCachedStat + "\n")
	}

	rgContext := search.RGContext(stagedFiles, opts.RGPattern)
	fdContext := search.FDContext(stagedFiles, opts.FDPattern)
	extraContext := mergeContext(rgContext, fdContext)

	var raw string
	if err := spinner.Run("正在调用 AI 生成 commit 消息（已暂存变更）", func() error {
		r, err := llm.Generate(ctx, ai.Request{
			Types:        typeList,
			Scopes:       scopeList,
			Diff:         stagedDiff,
			GitInfo:      gitInfo.String(),
			ExtraContext: extraContext,
		})
		if err != nil {
			return err
		}
		raw = r
		return nil
	}); err != nil {
		return fmt.Errorf("AI 生成消息失败: %w", err)
	}

	entries, reason := commit.ParseEntries(raw)
	if len(entries) == 0 {
		return fmt.Errorf("AI 未生成有效 commit 消息")
	}
	if reason != "" {
		fmt.Println(color.YellowS("\n" + reason))
	}

	var valid []commit.Entry
	for _, e := range entries {
		if e.Type == "" || e.Scope == "" || e.Message == "" {
			fmt.Fprintln(os.Stderr, color.YellowS("警告: AI 返回的 entry 缺少必要字段，已跳过"))
			continue
		}
		valid = append(valid, e)
	}
	entries = valid
	if len(entries) == 0 {
		return fmt.Errorf("AI 未生成有效 commit 消息")
	}

	if len(entries) == 1 {
		fmt.Println()
		fmt.Println(color.BoldCyanS("已暂存变更将作为 1 条提交"))
		fmt.Println()
	} else {
		fmt.Println(color.BoldCyanF("已暂存变更将分为 %d 条提交", len(entries)))
	}

	for i := range entries {
		entries[i].Type = strutil.Capitalize(entries[i].Type)
		entries[i].Scope = strutil.Capitalize(entries[i].Scope)
		msg := commit.FormatMessage(entries[i])

		if len(entries) > 1 {
			fmt.Println(color.CyanF("\n--- 已暂存提交 %d/%d ---", i+1, len(entries)))
		}

		if !commit.ConfirmEntry(entries[i]) {
			fmt.Println(color.YellowS("已取消"))
			return nil
		}

		if len(entries[i].Files) > 0 {
			if err := commit.CommitFiles(msg, entries[i].Files); err != nil {
				return fmt.Errorf("git commit 失败: %w", err)
			}
		} else {
			if err := commit.Commit(msg); err != nil {
				return fmt.Errorf("git commit 失败: %w", err)
			}
		}
	}
	fmt.Println(color.GreenS("已暂存变更提交成功"))
	return nil
}

func handleUnstagedChanges(ctx context.Context, llm *ai.OpenAI, typeList, scopeList []string, opts Options) error {
	var (
		diff           string
		changedFiles   []string
		statusShort    string
		diffStat       string
		diffCachedStat string
		untrackedFiles []string
	)

	if err := spinner.Run("正在获取未暂存 git 变更信息", func() error {
		d, err := git.Diff()
		if err != nil {
			return err
		}
		diff = d

		cf, err := git.ChangedFiles()
		if err != nil {
			fmt.Fprintln(os.Stderr, color.YellowF("警告: 获取变更文件列表失败: %v", err))
		}
		changedFiles = cf

		statusShort, _ = git.StatusShort()
		diffStat, _ = git.DiffStat()
		diffCachedStat, _ = git.DiffCachedStat()

		uf, _ := git.LsUntracked()
		untrackedFiles = uf
		return nil
	}); err != nil {
		return fmt.Errorf("获取 git 信息失败: %w", err)
	}

	if diff == "" && len(untrackedFiles) == 0 {
		fmt.Println(color.GrayS("没有未暂存的变更"))
		return nil
	}

	var gitInfo strings.Builder
	if statusShort != "" {
		gitInfo.WriteString("--- 工作区状态 ---\n" + statusShort + "\n")
	}
	if diffCachedStat != "" {
		gitInfo.WriteString("--- 已暂存变更摘要 ---\n" + diffCachedStat + "\n")
	}
	if diffStat != "" {
		gitInfo.WriteString("--- 未暂存变更摘要 ---\n" + diffStat + "\n")
	}
	if len(untrackedFiles) > 0 {
		untrackedContent := git.UntrackedContent(untrackedFiles)
		if untrackedContent != "" {
			gitInfo.WriteString("--- 未跟踪文件内容 ---\n" + untrackedContent + "\n")
		}
	}

	rgContext := search.RGContext(changedFiles, opts.RGPattern)
	fdContext := search.FDContext(changedFiles, opts.FDPattern)
	extraContext := mergeContext(rgContext, fdContext)

	var raw string
	if err := spinner.Run("正在调用 AI 生成 commit 消息", func() error {
		r, err := llm.Generate(ctx, ai.Request{
			Types:        typeList,
			Scopes:       scopeList,
			Diff:         diff,
			GitInfo:      gitInfo.String(),
			ExtraContext: extraContext,
		})
		if err != nil {
			return err
		}
		raw = r
		return nil
	}); err != nil {
		return fmt.Errorf("AI 生成消息失败: %w", err)
	}

	entries, reason := commit.ParseEntries(raw)
	if len(entries) == 0 {
		return fmt.Errorf("AI 未生成有效 commit 消息")
	}
	if reason != "" {
		fmt.Println(color.YellowS("\n" + reason))
	}

	var valid []commit.Entry
	for _, e := range entries {
		if e.Type == "" || e.Scope == "" || e.Message == "" {
			fmt.Fprintln(os.Stderr, color.YellowS("警告: AI 返回的 entry 缺少必要字段，已跳过"))
			continue
		}
		valid = append(valid, e)
	}
	entries = valid
	if len(entries) == 0 {
		return fmt.Errorf("AI 未生成有效 commit 消息")
	}

	if len(entries) == 1 {
		fmt.Println()
		fmt.Println(color.BoldCyanS("本次变更将作为 1 条提交"))
		fmt.Println()
	} else {
		fmt.Println(color.BoldCyanF("本次变更将分为 %d 条提交", len(entries)))
	}

	for i := range entries {
		entries[i].Type = strutil.Capitalize(entries[i].Type)
		entries[i].Scope = strutil.Capitalize(entries[i].Scope)
		msg := commit.FormatMessage(entries[i])

		if len(entries) > 1 {
			fmt.Println(color.CyanF("\n--- 提交 %d/%d ---", i+1, len(entries)))
		}

		if !commit.ConfirmEntry(entries[i]) {
			fmt.Println(color.YellowS("已取消"))
			return nil
		}

		if len(entries[i].Files) > 0 {
			if err := commit.StageFiles(entries[i].Files); err != nil {
				return fmt.Errorf("git add 失败: %w", err)
			}
		} else {
			if err := commit.Stage(); err != nil {
				return fmt.Errorf("git add 失败: %w", err)
			}
		}
		if err := commit.Commit(msg); err != nil {
			return fmt.Errorf("git commit 失败: %w", err)
		}
	}
	fmt.Println(color.GreenS("全部提交成功"))
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
