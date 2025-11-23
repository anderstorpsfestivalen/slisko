package patterns

import (
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/configuration"
	"github.com/anderstorpsfestivalen/slisko/pkg/traffic"
)

type Blink48Ports struct {
	ports []PortState
	style BlinkStyle
}

func (p *Blink48Ports) Render(info RenderInfo, c *chassi.Chassi) {
	for i := range p.ports {
		p.ports[i].Render()
	}
}

func (p *Blink48Ports) Info() PatternInfo {
	return PatternInfo{
		Name:     "blink48ports",
		Category: "link",
	}
}

func (p *Blink48Ports) Bootstrap(c *chassi.Chassi) {
	p.style = Cisco7609Style()

	// Get or create default traffic shaper
	shaper := GetTrafficShaper()
	if shaper == nil {
		// Fallback to default if not set
		shaper = traffic.NewShaper(configuration.DefaultTrafficShaper())
	}

	for _, card := range c.GetCardOfType("6478") {
		for _, port := range card.Link {
			time.Sleep(2 * time.Millisecond)
			p.ports = append(p.ports, p.style.CreatePort(port, shaper))
		}
	}
}
