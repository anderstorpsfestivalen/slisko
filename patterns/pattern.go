package patterns

import (
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
)

type PatternInfo struct {
	Name     string
	Category string
}

type RenderInfo struct {
	Start time.Time
}

type Pattern interface {
	Render(info RenderInfo, c *chassi.Chassi)
	Info() PatternInfo
	Bootstrap(c *chassi.Chassi)
}
