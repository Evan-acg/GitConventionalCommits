package strutil

import (
	"fmt"
	"regexp"
	"strings"
	"unicode"
)

var msgRe = regexp.MustCompile(`^(\w+)(\((\w+)\))?:\s*(.*)$`)

func Capitalize(s string) string {
	if s == "" {
		return ""
	}
	runes := []rune(s)
	runes[0] = unicode.ToUpper(runes[0])
	return string(runes)
}

func CapitalizeMessage(msg string) string {
	m := msgRe.FindStringSubmatch(strings.TrimSpace(msg))
	if m == nil {
		return msg
	}
	typeName := Capitalize(m[1])
	desc := m[4]
	if m[2] != "" {
		scope := Capitalize(m[3])
		return fmt.Sprintf("%s(%s): %s", typeName, scope, desc)
	}
	return fmt.Sprintf("%s: %s", typeName, desc)
}
