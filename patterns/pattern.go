package patterns

import "github.com/anderstorpsfestivalen/slisko/pkg/chassi"

type PatternInfo struct {
	Name     string
	Category string
}

type Pattern interface {
	Render(c *chassi.Chassi)
	Info() PatternInfo
}
