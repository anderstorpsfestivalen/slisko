package patterns

import (
	"math/rand"
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/faker"
	"github.com/anderstorpsfestivalen/slisko/pkg/pixel"
	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
)

type A9K8TL struct {
	bport []mapPort
	dead  []*pixel.Pixel
}

func (p *A9K8TL) Render(info RenderInfo, c *chassi.Chassi) {
	//for _, p := range c.GetCardOfType("6704") {
	//sys := utils.Square(math.Sin(utils.Random(0.00001, 0.01) * time.Since(info.Start).Seconds()))
	//p.Labeled["status"].SetClamped(0.0, sys, 0.0)

	//		}

	for _, p := range p.bport {
		if utils.Invert(p.faker.Trig()) == 1.0 {
			p.port.SetClamped(0.3, 1.0, 0.00)
		} else {
			p.port.SetClamped(1.0, 0.8, 0.00)
		}
	}
	for _, p := range p.dead {
		p.SetClamped(1.0, 0.0, 0.0)
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
			chance := rand.Intn(30) == 0
			if chance {
				p.dead = append(p.dead, c)
			} else {
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
}
