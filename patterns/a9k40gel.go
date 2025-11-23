package patterns

import (
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/configuration"
	"github.com/anderstorpsfestivalen/slisko/pkg/traffic"
)

type A9K40GE struct {
	ports []PortState
	style BlinkStyle
}

func (p *A9K40GE) Render(info RenderInfo, c *chassi.Chassi) {
	for i := range p.ports {
		p.ports[i].Render()
	}
}

func (p *A9K40GE) Info() PatternInfo {
	return PatternInfo{
		Name:     "a9k-40ge-l",
		Category: "misc",
	}
}

func (p *A9K40GE) Bootstrap(c *chassi.Chassi) {
	p.style = ASR9000Style()

	// Get or create default traffic shaper
	shaper := GetTrafficShaper()
	if shaper == nil {
		// Fallback to default if not set
		shaper = traffic.NewShaper(configuration.DefaultTrafficShaper())
	}

	for _, card := range c.GetCardOfType("A9K-40GE-L") {
		for _, port := range card.Link {
			time.Sleep(2 * time.Millisecond)
			p.ports = append(p.ports, p.style.CreatePort(port, shaper))
		}
	}
}
