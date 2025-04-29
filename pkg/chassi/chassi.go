package chassi

import "github.com/anderstorpsfestivalen/slisko/pkg/pixel"

type Chassi struct {
	LineCards []LineCard

	LEDs       []*pixel.Pixel
	LinkPorts  []*pixel.Pixel
	StatusLEDs []*pixel.Pixel
}

func CardsFromDefinition(s []string) []LineCard {
	chassi := []LineCard{}
	for _, v := range s {
		switch v {
		case "a9k-8t-l":
			chassi = append(chassi, GenA9K8T())
		case "blank":
			chassi = append(chassi, GenBlank())
		case "a9k-40ge-l":
			chassi = append(chassi, GenA9K40GE())
		case "a9k-rsp400-se":
			chassi = append(chassi, GenA9KRSP440SE())
		case "6478":
			chassi = append(chassi, Gen6478())
		case "6704":
			chassi = append(chassi, Gen6704())
		case "sup720":
			chassi = append(chassi, GenSUP720())
		}
	}
	return chassi
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
		if lc.Active {
			lp = append(lp, lc.Link...)
		}
	}
	return lp
}

func (c *Chassi) getstatusLEDs() []*pixel.Pixel {
	lp := []*pixel.Pixel{}
	for _, lc := range c.LineCards {
		if lc.Active {
			if lc.Status != nil {
				lp = append(lp, lc.Status)
			}
		}
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
	for i, lc := range c.LineCards {
		if lc.Name == ct {
			t = append(t, &c.LineCards[i])
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

func (c *Chassi) GetLEDsWithLabel(label string) []*pixel.Pixel {
	d := []*pixel.Pixel{}
	for i, _ := range c.LineCards {
		if val, ok := c.LineCards[i].Labeled[label]; ok {
			d = append(d, val)
		}
	}
	return d
}
