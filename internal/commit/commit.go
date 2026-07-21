package commit

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"strings"

	"github.com/Evan-acg/GitConventionalCommits/internal/color"
)

type Entry struct {
	Type    string   `json:"type"`
	Scope   string   `json:"scope"`
	Message string   `json:"message"`
	Files   []string `json:"files,omitempty"`
	Detail  string   `json:"detail,omitempty"`
}

type AIResponse struct {
	Reason string  `json:"reason"`
	Data   []Entry `json:"data"`
}

func ConfirmEntry(entry Entry) bool {
	msg := FormatMessage(entry)
	fmt.Println(color.CyanS("生成的提交消息:"))
	fmt.Println(color.BoldS(msg))
	if len(entry.Files) > 0 {
		fmt.Println(color.CyanS("\n关联文件:"))
		for _, f := range entry.Files {
			fmt.Println("  " + f)
		}
	}
	fmt.Print(color.CyanS("\n确认提交？回复 ok 执行: "))

	scanner := bufio.NewScanner(os.Stdin)
	scanner.Scan()
	return strings.TrimSpace(scanner.Text()) == "ok"
}

func FormatMessage(entry Entry) string {
	msg := entry.Type + "(" + entry.Scope + "): " + entry.Message
	if entry.Detail != "" {
		msg += "\n\n" + entry.Detail
	}
	return msg
}

func Stage() error {
	return exec.Command("git", "add", "-A").Run()
}

func StageFiles(files []string) error {
	args := append([]string{"add"}, files...)
	return exec.Command("git", args...).Run()
}

func UnstageFiles(files []string) error {
	args := append([]string{"restore", "--staged"}, files...)
	return exec.Command("git", args...).Run()
}

func Commit(msg string) error {
	parts := strings.SplitN(msg, "\n\n", 2)
	args := []string{"commit", "-m", parts[0]}
	if len(parts) > 1 && parts[1] != "" {
		args = append(args, "-m", parts[1])
	}
	cmd := exec.Command("git", args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

func CommitFiles(msg string, files []string) error {
	parts := strings.SplitN(msg, "\n\n", 2)
	args := []string{"commit", "-m", parts[0]}
	if len(parts) > 1 && parts[1] != "" {
		args = append(args, "-m", parts[1])
	}
	args = append(args, "--")
	args = append(args, files...)
	cmd := exec.Command("git", args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

func ParseEntries(raw string) ([]Entry, string) {
	raw = strings.TrimSpace(raw)
	if raw == "" {
		return nil, ""
	}

	var resp AIResponse
	if err := json.Unmarshal([]byte(raw), &resp); err != nil {
		return nil, ""
	}

	return resp.Data, resp.Reason
}
