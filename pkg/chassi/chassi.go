package chassi

import "github.com/anderstorpsfestivalen/slisko/pkg/pixel"

type Chassi struct {
	LineCards []LineCard

	LEDs       []*pixel.Pixel
	LinkPorts  []*pixel.Pixel
	StatusLEDs []*pixel.Pixel
}

func New(l []LineCard) Chassi {
	c := Chassi{
		LineCards: l,
	}

	c.LEDs = c.getleds()
	c.LinkPorts = c.getlinkPorts()
	c.StatusLEDs = c.getstatusLEDs()

	return c
}

func (c *Chassi) getlinkPorts() []*pixel.Pixel {
	lp := []*pixel.Pixel{}
	for _, lc := range c.LineCards {
		lp = append(lp, lc.Link...)
	}
	return lp
}

func (c *Chassi) getstatusLEDs() []*pixel.Pixel {
	lp := []*pixel.Pixel{}
	for _, lc := range c.LineCards {
		lp = append(lp, lc.Status)
	}
	return lp
}

func (c *Chassi) getleds() []*pixel.Pixel {
	lp := []*pixel.Pixel{}
	for v, lc := range c.LineCards {
		for z, _ := range lc.LEDs {
			lp = append(lp, &c.LineCards[v].LEDs[z])
		}
	}
	return lp
}

func (c *Chassi) GetCardOfType(ct string) []*LineCard {
	t := []*LineCard{}
	for _, lc := range c.LineCards {
		if lc.Name == ct {
			t = append(t, &lc)
		}
	}
	return t
}

func (c *Chassi) GetCardOrder() []string {
	st := []string{}
	for _, lc := range c.LineCards {
		st = append(st, lc.Name)
	}
	return st
}
