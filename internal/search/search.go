package search

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

var defaultPattern = "^(func|function|def|class|type|struct|interface|impl|fn|pub|export|const|let|var|module|trait|enum)\\b"

func Context(files []string, pattern string) string {
	if _, err := exec.LookPath("rg"); err != nil {
		fmt.Fprintln(os.Stderr, "建议安装 ripgrep (rg) 以获得更精准的 commit 消息")
		return ""
	}
	if len(files) == 0 {
		return ""
	}
	if pattern == "" {
		pattern = defaultPattern
	}

	args := []string{"--line-number", "-e", pattern}
	args = append(args, files...)

	out, err := exec.Command("rg", args...).Output()
	if err != nil {
		return ""
	}
	return strings.TrimSpace(string(out))
}

func RelatedFiles(changedFiles []string, pattern string) string {
	if _, err := exec.LookPath("fd"); err != nil {
		fmt.Fprintln(os.Stderr, "建议安装 fd (fd-find) 以获得更精准的 commit 消息")
		return ""
	}
	if len(changedFiles) == 0 {
		return ""
	}

	var sections []string

	if pattern != "" {
		out, err := exec.Command("fd", pattern).Output()
		if err == nil && len(out) > 0 {
			sections = append(sections, "自定义搜索结果:\n"+strings.TrimSpace(string(out)))
		}
	}

	seenStems := make(map[string]bool)
	for _, f := range changedFiles {
		stem := strings.TrimSuffix(filepath.Base(f), filepath.Ext(f))
		if seenStems[stem] {
			continue
		}
		seenStems[stem] = true

		out, err := exec.Command("fd", "--type", "f", "--glob", fmt.Sprintf("**/%s.*", stem)).Output()
		if err != nil || len(out) == 0 {
			continue
		}

		lines := strings.Split(strings.TrimSpace(string(out)), "\n")
		var related []string
		for _, line := range lines {
			if line != f {
				related = append(related, line)
			}
		}
		if len(related) > 0 {
			sections = append(sections, fmt.Sprintf("%s 的关联文件:\n%s", f, strings.Join(related, "\n")))
		}
	}

	seenDirs := make(map[string]bool)
	for _, f := range changedFiles {
		dir := filepath.Dir(f)
		if seenDirs[dir] {
			continue
		}
		seenDirs[dir] = true

		out, err := exec.Command("fd", "--type", "f", "--max-depth", "1", ".", dir).Output()
		if err != nil || len(out) == 0 {
			continue
		}

		lines := strings.Split(strings.TrimSpace(string(out)), "\n")
		var dirFiles []string
		for _, line := range lines {
			dirFiles = append(dirFiles, "  "+line)
		}
		sections = append(sections, fmt.Sprintf("%s/ 目录文件:\n%s", dir, strings.Join(dirFiles, "\n")))
	}

	return strings.Join(sections, "\n")
}
