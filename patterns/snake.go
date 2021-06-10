package patterns

import (
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
)

type Snake struct {
}

func (p *Snake) Render(info RenderInfo, c *chassi.Chassi) {
	//v := utils.Square(math.Sin(utils.Random(1, 5) * time.Since(info.Start).Seconds()))
	for i := 0; i < len(c.LEDs); i++ {
		for m, port := range c.LEDs {
			if m == i {
				port.SetClamped(128, 128, 128)
				time.Sleep(5 * time.Millisecond)
			} else {
				port.SetClamped(0, 0, 0)
				time.Sleep(5 * time.Millisecond)
			}
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
