package patterns

import (
	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
)

type Static struct {
}

func (p *Static) Render(info RenderInfo, c *chassi.Chassi) {
	// if p.snakeFrame > int64(len(c.LEDs)) {
	// 	p.snakeFrame = 0
	// } else {
	// 	p.snakeFrame = p.snakeFrame + 1
	// }

	//1 ÄR SPEED
	//l := math.Floor(utils.Sin(info.Start, 1) * float64(len(c.LEDs)))

	for _, port := range c.LEDs {

		port.SetClamped(1.0, 0.5, 0.5)

	}
}

func (p *Static) Info() PatternInfo {
	return PatternInfo{
		Name:     "static",
		Category: "global",
	}
}

func (p *Static) Bootstrap(c *chassi.Chassi) {
}
