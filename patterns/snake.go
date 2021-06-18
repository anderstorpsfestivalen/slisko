package patterns

import (
	"math"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
)

type Snake struct {
	snakeFrame int64
}

func (p *Snake) Render(info RenderInfo, c *chassi.Chassi) {
	// if p.snakeFrame > int64(len(c.LEDs)) {
	// 	p.snakeFrame = 0
	// } else {
	// 	p.snakeFrame = p.snakeFrame + 1
	// }

	//1 ÄR SPEED
	l := math.Floor(utils.Sin(info.Start, 1) * float64(len(c.LEDs)))

	for m, port := range c.LEDs {
		if m == int(l) {
			port.SetClamped(1.0, 1.0, 0.5)
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
