package patterns

import (
	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
)

type BlinkPorts struct {
}

func (p *BlinkPorts) Render(c *chassi.Chassi) {
	for _, p := range c.LinkPorts {
		p.SetClamped(1.0, 1.0, 0.3)
	}

}

func (p *BlinkPorts) Info() PatternInfo {
	return PatternInfo{
		Name:     "blinkports",
		Category: "link",
	}
}
