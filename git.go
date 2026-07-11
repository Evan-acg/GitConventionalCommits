package main

import (
	"os/exec"
)

func getGitDiff() (string, error) {
	out, err := exec.Command("git", "diff").Output()
	if err != nil {
		return "", err
	}
	return string(out), nil
}
