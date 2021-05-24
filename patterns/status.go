package patterns

import (
	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
)

type GreenStatus struct {
}

func (p *GreenStatus) Render(info RenderInfo, c *chassi.Chassi) {
	for _, p := range c.StatusLEDs {
		p.SetClamped(0.3, 1.0, 0.0)
	}
}

func (p *GreenStatus) Info() PatternInfo {
	return PatternInfo{
		Name:     "greenstatus",
		Category: "status",
	}
}

func (p *GreenStatus) Bootstrap() {

}

type RedStatus struct {
}

func (p *RedStatus) Render(info RenderInfo, c *chassi.Chassi) {
	for _, p := range c.StatusLEDs {
		p.SetClamped(1.0, 0.3, 0.0)
	}
}

func (p *RedStatus) Info() PatternInfo {
	return PatternInfo{
		Name:     "redstatus",
		Category: "status",
	}
}

func (p *RedStatus) Bootstrap() {

}
