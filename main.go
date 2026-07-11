package main

import (
	"flag"
	"fmt"
	"os"
)

func main() {
	skillPath := flag.String("skill-path", "", "path to git-commit SKILL.md")
	flag.Parse()

	if v := os.Getenv("GIT_COMMIT_SKILL_PATH"); v != "" && *skillPath == "" {
		*skillPath = v
	}

	typeList, scopeList := loadConfig(*skillPath)

	diff, err := getGitDiff()
	if err != nil {
		fmt.Fprintf(os.Stderr, "获取 diff 失败: %v\n", err)
		os.Exit(1)
	}
	if diff == "" {
		fmt.Println("没有未暂存的变更")
		return
	}

	msg, err := generateMessage(typeList, scopeList, diff)
	if err != nil {
		fmt.Fprintf(os.Stderr, "AI 生成消息失败: %v\n", err)
		os.Exit(1)
	}

	msg = enforceCapitalization(msg)

	if confirmCommit(msg) {
		if err := executeCommit(msg); err != nil {
			fmt.Fprintf(os.Stderr, "提交失败: %v\n", err)
			os.Exit(1)
		}
		fmt.Println("提交成功")
	} else {
		fmt.Println("已取消")
	}
}
