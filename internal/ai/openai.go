package ai

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
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

type OpenAI struct {
	APIKey  string
	Model   string
	BaseURL string
	Client  *http.Client
}

func (o *OpenAI) Generate(ctx context.Context, req Request) (string, error) {
	var typeList, scopeList string
	for _, t := range req.Types {
		typeList += fmt.Sprintf("- %s\n", t)
	}
	for _, s := range req.Scopes {
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

	userContent := "请根据以下 diff 生成 commit 消息:\n\n" + req.Diff
	if req.ExtraContext != "" {
		userContent += "\n---\n变更上下文:\n" + req.ExtraContext
	}

	body, _ := json.Marshal(chatRequest{
		Model: o.Model,
		Messages: []chatMessage{
			{Role: "system", Content: systemPrompt},
			{Role: "user", Content: userContent},
		},
	})

	url := strings.TrimSuffix(o.BaseURL, "/") + "/v1/chat/completions"
	httpReq, err := http.NewRequestWithContext(ctx, "POST", url, bytes.NewReader(body))
	if err != nil {
		return "", fmt.Errorf("创建请求失败: %w", err)
	}
	httpReq.Header.Set("Authorization", "Bearer "+o.APIKey)
	httpReq.Header.Set("Content-Type", "application/json")

	client := o.Client
	if client == nil {
		client = http.DefaultClient
	}

	resp, err := client.Do(httpReq)
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
