package patterns

import (
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/faker"
	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
)

type A9K8TL struct {
	bport []mapPort
}

func (p *A9K8TL) Render(info RenderInfo, c *chassi.Chassi) {
	//for _, p := range c.GetCardOfType("6704") {
	//sys := utils.Square(math.Sin(utils.Random(0.00001, 0.01) * time.Since(info.Start).Seconds()))
	//p.Labeled["status"].SetClamped(0.0, sys, 0.0)

	//		}

	for _, p := range p.bport {
		v := utils.Invert(p.faker.Trig())
		p.port.SetClamped(v*0.3, v*1.0, v*0.00)
	}
}

func (p *A9K8TL) Info() PatternInfo {
	return PatternInfo{
		Name:     "a9k-8t-l",
		Category: "misc",
	}
}

func (p *A9K8TL) Bootstrap(c *chassi.Chassi) {
	for _, card := range c.GetCardOfType("A9K-8T-L") {
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
