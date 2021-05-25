package patterns

import (
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/faker"
	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
)

type Blink48Ports struct {
	bport []faker.Fake
}

func (p *Blink48Ports) Render(info RenderInfo, c *chassi.Chassi) {
	// v := utils.Square(math.Sin(20 * time.Since(info.Start).Seconds()))
	// for _, p := range c.LinkPorts {
	// 	p.SetClamped(v, v*0.7, 0.0)
	// }

	for i, card := range c.GetCardOfType("6478") {
		for k, port := range card.Link {
			v := utils.Invert(p.bport[i+k].Trig())
			port.SetClamped(v, v*0.7, 0.0)
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
			p.bport = append(p.bport, faker.NewRandomInterval(100*time.Millisecond, 2*time.Second, 100*time.Millisecond, 12000*time.Millisecond,
				faker.NewRandomBlinker(30, 40, 10*time.Millisecond, 5*time.Second)))
		}
	}

}
