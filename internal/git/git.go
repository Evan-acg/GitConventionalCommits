package git

import (
	"os/exec"
)

func Diff() (string, error) {
	out, err := exec.Command("git", "diff").Output()
	if err != nil {
		return "", err
	}
	return string(out), nil
}
