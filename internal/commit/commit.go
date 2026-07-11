package commit

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"strings"
)

func Confirm(msg string) bool {
	fmt.Println("\n生成的提交消息:")
	fmt.Println(msg)
	fmt.Print("\n确认提交？回复 ok 执行: ")

	scanner := bufio.NewScanner(os.Stdin)
	scanner.Scan()
	return strings.TrimSpace(scanner.Text()) == "ok"
}

func Stage() error {
	return exec.Command("git", "add", "-A").Run()
}

func Commit(msg string) error {
	cmd := exec.Command("git", "commit", "-m", msg)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}
