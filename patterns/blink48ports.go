package patterns

import (
	"math/rand"
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/faker"
	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
)

type Blink48Ports struct {
	bport []ColorMap
}

type ColorMap struct {
	faker faker.Fake
	speed bool
}

func (p *Blink48Ports) Render(info RenderInfo, c *chassi.Chassi) {
	// v := utils.Square(math.Sin(20 * time.Since(info.Start).Seconds()))
	// for _, p := range c.LinkPorts {
	// 	p.SetClamped(v, v*0.7, 0.0)
	// }

	for i, card := range c.GetCardOfType("6478") {
		for k, port := range card.Link {
			v := utils.Invert(p.bport[(i+1)*k].faker.Trig())
			if p.bport[(i+1)*k].speed {
				port.SetClamped(v*0.5, v*1.0, v*0.3)
			} else {
				port.SetClamped(v*1.0, v*1.0, v*0.3)
			}
		}
	}

}

func (p *Blink48Ports) Info() PatternInfo {
	return PatternInfo{
		Name:     "blink48ports",
		Category: "link",
	}
}

func (p *Blink48Ports) Bootstrap(c *chassi.Chassi) {
	for _, card := range c.GetCardOfType("6478") {
		for range card.Link {
			rand.Seed(time.Now().UTC().UnixNano())
			time.Sleep(2 * time.Millisecond)
			r := rand.Intn(1000)
			speed := true
			if r > 900 {
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
				})
		}
	}

}
