package ai

import "context"

type Request struct {
	Types        []string
	Scopes       []string
	Diff         string
	GitInfo      string
	ExtraContext string
}

type Provider interface {
	Generate(ctx context.Context, req Request) (string, error)
}
