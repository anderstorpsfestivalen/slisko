package patterns

import (
	"math/rand"
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/faker"
	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
)

type A9K40GE struct {
	bport []ColorMap
}

func (p *A9K40GE) Render(info RenderInfo, c *chassi.Chassi) {
	//for _, p := range c.GetCardOfType("6704") {
	//sys := utils.Square(math.Sin(utils.Random(0.00001, 0.01) * time.Since(info.Start).Seconds()))
	//p.Labeled["status"].SetClamped(0.0, sys, 0.0)

	//		}

	for _, p := range p.bport {
		v := utils.Invert(p.faker.Trig())
		p.port.SetClamped(v*0.3, v*1.0, v*0.00)
	}
}

func (p *A9K40GE) Info() PatternInfo {
	return PatternInfo{
		Name:     "a9k-40ge-l",
		Category: "misc",
	}
}

func (p *A9K40GE) Bootstrap(c *chassi.Chassi) {
	for _, card := range c.GetCardOfType("A9K-40GE-L") {
		for _, pb := range card.Link {
			time.Sleep(2 * time.Millisecond)
			r := rand.Intn(100)
			speed := true
			if r > 80 {
				speed = false
			}
			p.bport = append(p.bport,
				ColorMap{
					faker: faker.NewRandomInterval(
						100*time.Millisecond,
						7*time.Second,
						100*time.Millisecond,
						12*time.Second,
						faker.NewRandomBlinker(15, 40, 1*time.Second, 10*time.Second),
					),
					speed: speed,
					port:  pb,
				})
		}
	}
}
