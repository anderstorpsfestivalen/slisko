package patterns

import (
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/faker"
	"github.com/anderstorpsfestivalen/slisko/pkg/pixel"
)

type PatternInfo struct {
	Name     string
	Category string
}

type RenderInfo struct {
	Start time.Time
	Frame int64
}

type Pattern interface {
	Render(info RenderInfo, c *chassi.Chassi)
	Info() PatternInfo
	Bootstrap(c *chassi.Chassi)
}

type mapPort struct {
	faker faker.Fake
	port  *pixel.Pixel
}
