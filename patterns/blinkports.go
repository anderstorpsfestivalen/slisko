package patterns

import (
	"math"
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
)

type BlinkPorts struct {
}

func (p *BlinkPorts) Render(info RenderInfo, c *chassi.Chassi) {
	v := utils.Square(math.Sin(20 * time.Since(info.Start).Seconds()))
	for _, p := range c.LinkPorts {
		p.SetClamped(v, v*0.7, 0.0)
	}

}

func (p *BlinkPorts) Info() PatternInfo {
	return PatternInfo{
		Name:     "blinkports",
		Category: "link",
	}
}

func (p *BlinkPorts) Bootstrap() {

}
