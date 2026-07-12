package git

import (
	"bytes"
	"fmt"
	"os"
	"os/exec"
	"strings"
)

func Diff() (string, error) {
	out, err := exec.Command("git", "diff").Output()
	if err != nil {
		return "", err
	}
	return string(out), nil
}

func ChangedFiles() ([]string, error) {
	out, err := exec.Command("git", "diff", "--name-only").Output()
	if err != nil {
		return nil, err
	}
	files := strings.Fields(string(out))
	return files, nil
}

func StatusShort() (string, error) {
	out, err := exec.Command("git", "status", "--short").Output()
	if err != nil {
		return "", err
	}
	return string(out), nil
}

func DiffStat() (string, error) {
	out, err := exec.Command("git", "diff", "--stat").Output()
	if err != nil {
		return "", err
	}
	return string(out), nil
}

func DiffCachedStat() (string, error) {
	out, err := exec.Command("git", "diff", "--cached", "--stat").Output()
	if err != nil {
		return "", err
	}
	return string(out), nil
}

func LsUntracked() ([]string, error) {
	out, err := exec.Command("git", "ls-files", "--others", "--exclude-standard").Output()
	if err != nil {
		return nil, err
	}
	files := strings.Fields(string(out))
	return files, nil
}

func UntrackedContent(paths []string) string {
	var sb strings.Builder
	for _, path := range paths {
		info, err := os.Stat(path)
		if err != nil {
			continue
		}
		if info.Size() > 100*1024 {
			sb.WriteString(fmt.Sprintf("--- 未跟踪文件 (超过100KB，已跳过): %s ---\n", path))
			continue
		}
		data, err := os.ReadFile(path)
		if err != nil {
			continue
		}
		if bytes.Contains(data, []byte{0}) {
			sb.WriteString(fmt.Sprintf("--- 未跟踪文件 (二进制): %s ---\n", path))
			continue
		}
		sb.WriteString(fmt.Sprintf("--- 未跟踪文件: %s ---\n%s\n", path, string(data)))
	}
	return sb.String()
}
