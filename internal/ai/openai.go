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
	Error *struct {
		Message string `json:"message"`
		Type    string `json:"type"`
	} `json:"error"`
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
		`你是一个 git commit 消息生成助手。根据以下 git 信息和可用的 type/scope 分类，生成 conventional commit 消息。

可用的 Type:
%s

可用的 Scope:
%s

	输出格式: 始终返回 JSON 对象，包含以下字段:
- reason: 分析说明 (中文，说明为何选择单条或多条提交)
- data: commit 消息数组，每个元素包含以下字段:
  - type: 变更类型 (必填，从可用 Type 中选择)
  - scope: 变更范围 (必填，从可用 Scope 中选择)
  - message: 中文描述 (一句话概括变更内容)
  - files: 该 commit 涉及的文件路径数组 (需要 git add 的文件)
  - detail: 变更的详细描述 (markdown 列表格式，以 - 开头列出每个具体变更)

规则:
- 根据 diff 内容选择最匹配的 Type 和 Scope
- type 和 scope 为必填字段，不得为空
- 消息用中文描述变更内容
- 分析 diff 内容判断是否需要分多条 commit
- 如果 diff 包含多个独立不相关的变更，为每组独立变更输出一条 commit
- 如果所有变更是相关的、完成单一目标，只输出一条
- 只返回 JSON 对象，不要额外说明

示例输出:
{"reason": "本次变更中的修改紧密相关，适合作为单条提交", "data": [{"type": "Feat", "scope": "Git", "message": "添加新的 git 函数", "files": ["internal/git/git.go"], "detail": "- 新增 StatusShort 函数\n- 新增 DiffStat 函数"}]}`, typeList, scopeList)

	userContent := "请根据以下 git 信息生成 commit 消息:\n\n"
	if req.GitInfo != "" {
		userContent += "--- 工作区状态 ---\n" + req.GitInfo + "\n\n"
	}
	userContent += "--- 完整 diff ---\n" + req.Diff
	if req.ExtraContext != "" {
		userContent += "\n\n--- 变更上下文 ---\n" + req.ExtraContext
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
		if chatResp.Error != nil {
			return "", fmt.Errorf("API 返回错误: %s (%s)", chatResp.Error.Message, chatResp.Error.Type)
		}
		return "", fmt.Errorf("API 返回空结果（无错误信息）")
	}

	return strings.TrimSpace(chatResp.Choices[0].Message.Content), nil
}
