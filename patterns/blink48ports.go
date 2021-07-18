package patterns

import (
	"math/rand"
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/faker"
	"github.com/anderstorpsfestivalen/slisko/pkg/pixel"
	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
)

type Blink48Ports struct {
	bport []ColorMap
}

type ColorMap struct {
	faker faker.Fake
	speed bool
	port  *pixel.Pixel
}

func (p *Blink48Ports) Render(info RenderInfo, c *chassi.Chassi) {

	for _, port := range p.bport {
		v := utils.Invert(port.faker.Trig())
		if port.speed {
			port.port.SetClamped(v*0.5, v*1.0, v*0.3)
		} else {
			port.port.SetClamped(v*1.0, v*1.0, v*0.3)
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
		for _, pb := range card.Link {
			rand.Seed(time.Now().UTC().UnixNano())
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
