package patterns

import (
	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
	"github.com/lucasb-eyer/go-colorful"
)

type Colorcycler struct {
	cycle float64
	color colorful.Color
}

func (p *Colorcycler) Render(info RenderInfo, c *chassi.Chassi) {

	//v := (math.Sin(time.Since(info.Start).Seconds()) + 1) / 2 * 360
	//v := utils.Triangle(info.Start, 2, 1.0) * 360
	v := utils.Sin(info.Start, 0.2) * 360
	p.color = colorful.Hsv(v, 1.0, 1.0)
	for _, port := range c.LEDs {
		port.SetClamped(p.color.R, p.color.G, p.color.B)
	}

}

func (p *Colorcycler) Info() PatternInfo {
	return PatternInfo{
		Name:     "colorcycler",
		Category: "global",
	}
}

func (p *Colorcycler) Bootstrap(c *chassi.Chassi) {
	p.color = colorful.Color{0.313725, 0.478431, 0.721569}
}
