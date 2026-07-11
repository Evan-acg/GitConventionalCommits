package main

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"regexp"
	"strings"
	"unicode"
)

var commitMsgRe = regexp.MustCompile(`^(\w+)(\((\w+)\))?:\s*(.*)$`)

func capitalize(s string) string {
	if s == "" {
		return ""
	}
	runes := []rune(s)
	runes[0] = unicode.ToUpper(runes[0])
	return string(runes)
}

func enforceCapitalization(msg string) string {
	m := commitMsgRe.FindStringSubmatch(strings.TrimSpace(msg))
	if m == nil {
		return msg
	}
	typeName := capitalize(m[1])
	desc := m[4]
	if m[2] != "" {
		scope := capitalize(m[3])
		return fmt.Sprintf("%s(%s): %s", typeName, scope, desc)
	}
	return fmt.Sprintf("%s: %s", typeName, desc)
}

func executeCommit(msg string) error {
	if err := exec.Command("git", "add", "-A").Run(); err != nil {
		return fmt.Errorf("git add 失败: %w", err)
	}
	cmd := exec.Command("git", "commit", "-m", msg)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

func confirmCommit(msg string) bool {
	fmt.Println("\n生成的提交消息:")
	fmt.Println(msg)
	fmt.Print("\n确认提交？回复 ok 执行: ")

	scanner := bufio.NewScanner(os.Stdin)
	scanner.Scan()
	return strings.TrimSpace(scanner.Text()) == "ok"
}
