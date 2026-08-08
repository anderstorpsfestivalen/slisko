package patterns

import (
	"math"
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
)

type Strobe struct {
}

func (p *Strobe) Render(info RenderInfo, c *chassi.Chassi) {
	v := utils.Square(math.Sin(100 * time.Since(info.Start).Seconds()))
	for _, p := range c.LEDs {
		p.SetClamped(v, v, v)
	}

}

func (p *Strobe) Info() PatternInfo {
	return PatternInfo{
		Name:     "strobe",
		Category: "global",
	}
}

func (p *Strobe) Bootstrap(c *chassi.Chassi) {

}
