package patterns

import (
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/faker"
	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
)

type SUP720 struct {
	disk0 *faker.RandomInterval
	disk1 *faker.RandomInterval

	port0 faker.Fake
	port1 faker.Fake
}

func (p *SUP720) Render(info RenderInfo, c *chassi.Chassi) {
	for _, port := range c.GetCardOfType("sup720") {
		//sys := utils.Square(math.Sin(utils.Random(0.00001, 0.01) * time.Since(info.Start).Seconds()))
		port.Labeled["system"].SetClamped(0.2, 1.0, 0.0)

		port.Labeled["active"].SetClamped(0.2, 1.0, 0.0)
		port.Labeled["mgmt"].SetClamped(1.0, 0.0, 0.0)

		port.Labeled["disk0"].SetClamped(0.0, p.disk0.Trig(), 0.0)

		//disk1 := utils.Square(math.Sin(utils.Random(1, 5) * time.Since(info.Start).Seconds()))
		//disk1 := utils.Square(math.Sin(20 * time.Since(info.Start).Seconds()))
		port.Labeled["disk1"].SetClamped(0.0, p.disk1.Trig(), 0.0)

		p0 := utils.Invert(p.port0.Trig())
		port.Labeled["p1"].SetClamped(0.7*p0, 0.5*p0, 0.0*p0)

		p1 := utils.Invert(p.port1.Trig())
		port.Labeled["p2"].SetClamped(0.7*p1, 0.5*p1, 0.*p1)

	}
}

func (p *SUP720) Info() PatternInfo {
	return PatternInfo{
		Name:     "sup720",
		Category: "misc",
	}
}

func (p *SUP720) Bootstrap(c *chassi.Chassi) {
	p.disk0 = faker.NewRandomInterval(40*time.Second,
		12000*time.Second,
		100*time.Millisecond,
		6500*time.Millisecond,
		faker.NewRandomBlinker(15, 40, 1*time.Second, 10*time.Second))

	p.disk1 = faker.NewRandomInterval(40*time.Second,
		12000*time.Second,
		100*time.Millisecond,
		6500*time.Millisecond,
		faker.NewRandomBlinker(15, 40, 1*time.Second, 10*time.Second))

	p.port0 = faker.NewRandomInterval(
		300*time.Millisecond,
		12000*time.Millisecond,
		70*time.Millisecond,
		6500*time.Millisecond,
		faker.NewRandomBlinker(15, 40, 1*time.Second, 10*time.Second),
	)

	p.port1 = faker.NewRandomInterval(
		200*time.Millisecond,
		7*time.Second,
		70*time.Millisecond,
		12*time.Second,
		faker.NewRandomBlinker(15, 40, 1*time.Second, 10*time.Second),
	)

}
