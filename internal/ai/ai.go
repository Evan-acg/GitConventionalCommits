package ai

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"strings"
)

type chatMessage struct {
	Role    string `json:"role"`
	Content string `json:"content"`
}

type chatRequest struct {
	Model    string        `json:"model"`
	Messages []chatMessage `json:"messages"`
}

type chatResponse struct {
	Choices []struct {
		Message chatMessage `json:"message"`
	} `json:"choices"`
}

func Generate(types, scopes []string, diff string, apiKey string, rgContext string) (string, error) {
	if apiKey == "" {
		apiKey = os.Getenv("MESSAGE_API_KEY")
	}
	if apiKey == "" {
		return "", fmt.Errorf("MESSAGE_API_KEY 未设置，可通过 --api-key 参数或 MESSAGE_API_KEY 环境变量设置")
	}

	model := os.Getenv("OPENAI_MODEL")
	if model == "" {
		model = "gpt-4o-mini"
	}

	var typeList, scopeList string
	for _, t := range types {
		typeList += fmt.Sprintf("- %s\n", t)
	}
	for _, s := range scopes {
		scopeList += fmt.Sprintf("- %s\n", s)
	}

	systemPrompt := fmt.Sprintf(
		`你是一个 git commit 消息生成助手。根据 git diff 和可用的 type/scope 分类，生成一条 conventional commit 消息。

格式: Type(Scope): 中文消息

可用的 Type:
%s

可用的 Scope:
%s

规则:
- 根据 diff 内容选择最匹配的 Type 和 Scope
- 消息用中文描述变更内容
- 只返回一行消息，不要额外说明`, typeList, scopeList)

	userContent := fmt.Sprintf("请根据以下 diff 生成 commit 消息:\n\n%s", diff)
	if rgContext != "" {
		userContent += fmt.Sprintf("\n---\n变更文件结构上下文:\n%s", rgContext)
	}

	reqBody := chatRequest{
		Model: model,
		Messages: []chatMessage{
			{Role: "system", Content: systemPrompt},
			{Role: "user", Content: userContent},
		},
	}

	body, _ := json.Marshal(reqBody)

	req, err := http.NewRequest("POST", "https://api.openai.com/v1/chat/completions", bytes.NewReader(body))
	if err != nil {
		return "", fmt.Errorf("创建请求失败: %w", err)
	}
	req.Header.Set("Authorization", "Bearer "+apiKey)
	req.Header.Set("Content-Type", "application/json")

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return "", fmt.Errorf("API 请求失败: %w", err)
	}
	defer resp.Body.Close()

	var chatResp chatResponse
	if err := json.NewDecoder(resp.Body).Decode(&chatResp); err != nil {
		return "", fmt.Errorf("解析响应失败: %w", err)
	}

	if len(chatResp.Choices) == 0 {
		return "", fmt.Errorf("API 返回空结果")
	}

	return strings.TrimSpace(chatResp.Choices[0].Message.Content), nil
}
