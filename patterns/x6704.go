package patterns

import (
	"math"
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
)

type X6704 struct {
}

func (p *X6704) Render(info RenderInfo, c *chassi.Chassi) {
	for _, p := range c.GetCardOfType("x6704") {
		sys := utils.Square(math.Sin(utils.Random(0.00001, 0.01) * time.Since(info.Start).Seconds()))
		p.Labeled["status"].SetClamped(0.0, sys, 0.0)

		blink := utils.Square(math.Sin(utils.Random(1, 5) * time.Since(info.Start).Seconds()))
		p.Labeled["port0"].SetClamped(0.0, blink, 0.0)
		p.Labeled["port1"].SetClamped(0.0, blink, 0.0)
		p.Labeled["port2"].SetClamped(0.0, blink, 0.0)
		p.Labeled["port3"].SetClamped(0.0, blink, 0.0)

	}
}

func (p *X6704) Info() PatternInfo {
	return PatternInfo{
		Name:     "x6704",
		Category: "misc",
	}
}

func (p *X6704) Bootstrap() {}
