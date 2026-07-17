package color

import "fmt"

const (
	Reset  = "\033[0m"
	Bold   = "\033[1m"
	Red    = "\033[31m"
	Green  = "\033[32m"
	Yellow = "\033[33m"
	Cyan   = "\033[36m"
	Gray   = "\033[90m"
)

func S(s string) string    { return s }
func RedS(s string) string  { return Red + s + Reset }
func GreenS(s string) string { return Green + s + Reset }
func YellowS(s string) string { return Yellow + s + Reset }
func CyanS(s string) string  { return Cyan + s + Reset }
func GrayS(s string) string  { return Gray + s + Reset }
func BoldS(s string) string  { return Bold + s + Reset }

func RedF(format string, a ...any) string   { return Red + fmt.Sprintf(format, a...) + Reset }
func GreenF(format string, a ...any) string { return Green + fmt.Sprintf(format, a...) + Reset }
func YellowF(format string, a ...any) string { return Yellow + fmt.Sprintf(format, a...) + Reset }
func CyanF(format string, a ...any) string  { return Cyan + fmt.Sprintf(format, a...) + Reset }
func GrayF(format string, a ...any) string  { return Gray + fmt.Sprintf(format, a...) + Reset }
func BoldF(format string, a ...any) string  { return Bold + fmt.Sprintf(format, a...) + Reset }
