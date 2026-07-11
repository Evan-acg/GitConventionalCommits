package search

import (
	"fmt"
	"os"
	"os/exec"
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
