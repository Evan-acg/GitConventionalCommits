package spinner

import (
	"fmt"
	"os"
	"strings"
	"sync"
	"time"

	"github.com/Evan-acg/GitConventionalCommits/internal/color"
)

func Run(msg string, fn func() error) error {
	stop := make(chan struct{})
	var wg sync.WaitGroup
	wg.Add(1)

	go func() {
		defer wg.Done()
		chars := []rune("-/|\\")
		i := 0
		for {
			select {
			case <-stop:
				fmt.Fprintf(os.Stderr, "\r%s\r", strings.Repeat(" ", len(msg)+4))
				fmt.Fprintln(os.Stderr, msg+" "+color.GreenS("✓ 完成"))
				return
			default:
				fmt.Fprintf(os.Stderr, "\r%s %c", msg, chars[i%len(chars)])
				i++
				time.Sleep(100 * time.Millisecond)
			}
		}
	}()

	err := fn()
	close(stop)
	wg.Wait()
	return err
}
