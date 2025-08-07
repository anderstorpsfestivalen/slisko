package patterns

import (
	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
)

type Mapper struct {
}

func (p *Mapper) Render(info RenderInfo, c *chassi.Chassi) {
	// v := utils.Square(math.Sin(20 * time.Since(info.Start).Seconds()))
	// for _, p := range c.LinkPorts {
	// 	p.SetClamped(v, v*0.7, 0.0)
	// }
	/*
		v := c.GetCardOfType("6478")[0]
		v.Status.G = 1.0

		v = c.GetCardOfType("6704")[0]
		v.Status.G = 1.0

		v = c.GetCardOfType("6704")[1]
		v.Status.G = 1.0

		v = c.GetCardOfType("sup720")[0]
		v.Status.G = 1.0

		v = c.GetCardOfType("6704")[2]
		v.Status.G = 1.0

		v = c.GetCardOfType("6704")[3]
		v.Status.G = 1.0

		v = c.GetCardOfType("6478")[1]
		v.Status.G = 1.0

	*/

	// 40ge 0
	{
		v := c.GetCardOfType("A9K-40GE-L")[0]
		v.Status.G = 1.0
		for _, pb := range v.Link {
			pb.SetClamped(1.0, 0.0, 0.0)
		}
	}

	// a9k-8t-l
	{

		v := c.GetCardOfType("A9K-8T-L")[0]
		v.Status.G = 1.0
		for _, pb := range v.Link {
			pb.SetClamped(1.0, 0.0, 0.0)
		}
	}

	// a9k-8t-l
	{

		v := c.GetCardOfType("A9K-8T-L")[1]
		v.Status.G = 1.0
		for _, pb := range v.Link {
			pb.SetClamped(1.0, 0.0, 0.0)
		}
	}

	// rsp-440
	{

		v := c.GetCardOfType("A9K-RSP440-SE")[0]
		v.Labeled["gps"].G = 1.0

	}

	// rsp-440
	{

		v := c.GetCardOfType("A9K-RSP440-SE-2")[0]
		v.LEDs[10].G = 1.0
		//v.Labeled["fail"].G = 1.0

	}
	// a9k-8t-l
	{

		v := c.GetCardOfType("A9K-8T-L")[2]
		v.Status.G = 1.0
		for _, pb := range v.Link {
			pb.SetClamped(1.0, 0.0, 0.0)
		}
	}

	// a9k-8t-l
	{

		v := c.GetCardOfType("A9K-8T-L")[3]
		v.Status.G = 1.0
		for _, pb := range v.Link {
			pb.SetClamped(1.0, 0.0, 0.0)
		}
	}

	{
		v := c.GetCardOfType("A9K-40GE-L")[1]
		v.Status.G = 1.0
		for _, pb := range v.Link {
			pb.SetClamped(1.0, 0.0, 0.0)
		}
	}

}

func (p *Mapper) Info() PatternInfo {
	return PatternInfo{
		Name:     "mapper",
		Category: "global",
	}
}

func (p *Mapper) Bootstrap(c *chassi.Chassi) {

}
