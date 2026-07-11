package git

import (
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
