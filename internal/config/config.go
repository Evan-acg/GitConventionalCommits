package config

import (
	"os"
	"regexp"
	"strings"
	"unicode"

	"gopkg.in/yaml.v3"
)

type typeScopeItem struct {
	Name string `yaml:"name"`
	Docs string `yaml:"docs"`
}

type lazyGitConfig struct {
	Types  []typeScopeItem `yaml:"type"`
	Scopes []typeScopeItem `yaml:"scope"`
}

func Load(skillPath string) (types, scopes []string) {
	if data, err := os.ReadFile(".lazygit.yaml"); err == nil {
		var cfg lazyGitConfig
		if err := yaml.Unmarshal(data, &cfg); err == nil && len(cfg.Types) > 0 {
			for _, t := range cfg.Types {
				types = append(types, t.Name)
			}
			for _, s := range cfg.Scopes {
				scopes = append(scopes, s.Name)
			}
			return
		}
	}

	if skillPath != "" {
		if data, err := os.ReadFile(skillPath); err == nil {
			re := regexp.MustCompile("\\| `([^`]+)` \\|")
			matches := re.FindAllStringSubmatch(string(data), -1)
			for _, m := range matches {
				name := strings.TrimSpace(m[1])
				if name != "" {
					runes := []rune(name)
					runes[0] = unicode.ToUpper(runes[0])
					types = append(types, string(runes))
				}
			}
			if len(types) > 0 {
				return
			}
		}
	}

	types = []string{"Feat", "Fix", "Docs", "Style", "Refactor", "Perf", "Test", "Build", "CI", "Chore", "Revert"}
	return
}
