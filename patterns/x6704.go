package patterns

import (
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/faker"
	"github.com/anderstorpsfestivalen/slisko/pkg/pixel"
	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
)

type X6704 struct {
	bport []mapPort
}

type mapPort struct {
	faker faker.Fake
	port  *pixel.Pixel
}

func (p *X6704) Render(info RenderInfo, c *chassi.Chassi) {
	//for _, p := range c.GetCardOfType("6704") {
	//sys := utils.Square(math.Sin(utils.Random(0.00001, 0.01) * time.Since(info.Start).Seconds()))
	//p.Labeled["status"].SetClamped(0.0, sys, 0.0)

	//		}

	for _, p := range p.bport {
		v := utils.Invert(p.faker.Trig())
		p.port.SetClamped(v*0.3, v*1.0, v*0.00)
	}
}

func (p *X6704) Info() PatternInfo {
	return PatternInfo{
		Name:     "x6704",
		Category: "misc",
	}
}

func (p *X6704) Bootstrap(c *chassi.Chassi) {
	for _, card := range c.GetCardOfType("6704") {
		for _, c := range card.Link {
			p.bport = append(p.bport, mapPort{
				faker: faker.NewRandomInterval(
					100*time.Millisecond,
					7*time.Second,
					100*time.Millisecond,
					12*time.Second,
					faker.NewRandomBlinker(15, 40, 1*time.Second, 10*time.Second),
				),
				port: c,
			})
		}
	}
}
