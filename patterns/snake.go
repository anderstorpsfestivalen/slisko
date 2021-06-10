package patterns

import (
	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
)

type Snake struct {
	snakeFrame int64
}

func (p *Snake) Render(info RenderInfo, c *chassi.Chassi) {
	if p.snakeFrame > int64(len(c.LEDs)) {
		p.snakeFrame = 0
	} else {
		p.snakeFrame = p.snakeFrame + 1
	}

	for m, port := range c.LEDs {
		if m == int(p.snakeFrame) {
			port.SetClamped(0.5, 0.5, 0.5)
		} else {
			port.SetClamped(0.0, 0.0, 0.0)
		}
	}
}

func (p *Snake) Info() PatternInfo {
	return PatternInfo{
		Name:     "snake",
		Category: "global",
	}
}

func (p *Snake) Bootstrap(c *chassi.Chassi) {

}
