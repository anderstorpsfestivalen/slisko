package patterns

import (
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/faker"
)

type RSP440 struct {
	disk0 *faker.RandomInterval
	disk1 *faker.RandomInterval
}

func (p *RSP440) Render(info RenderInfo, c *chassi.Chassi) {

	for _, port := range c.GetCardOfType("A9K-RSP440-SE") {
		//sys := utils.Square(math.Sin(utils.Random(0.00001, 0.01) * time.Since(info.Start).Seconds()))
		port.Labeled["gps"].SetClamped(1.0, 0.0, 0.0)

		port.Labeled["sync"].SetClamped(1.0, 0.0, 0.0)

		port.Labeled["maj"].SetClamped(0.0, 1.0, 0.0)
		port.Labeled["min"].SetClamped(1.0, 0.5, 0.0)

		port.Labeled["sso"].SetClamped(0.0, 0.0, p.disk0.Trig())

	}

	for _, port := range c.GetCardOfType("A9K-RSP440-SE-2") {
		//sys := utils.Square(math.Sin(utils.Random(0.00001, 0.01) * time.Since(info.Start).Seconds()))
		port.Labeled["gps"].SetClamped(1.0, 0.0, 0.0)

		port.Labeled["sync"].SetClamped(1.0, 0.0, 0.0)

		port.Labeled["maj"].SetClamped(0.0, 1.0, 0.0)
		port.Labeled["min"].SetClamped(1.0, 0.5, 0.0)

		port.Labeled["sso"].SetClamped(0.0, 0.0, p.disk1.Trig())

	}
}

func (p *RSP440) Info() PatternInfo {
	return PatternInfo{
		Name:     "rsp440",
		Category: "misc",
	}
}

func (p *RSP440) Bootstrap(c *chassi.Chassi) {
	p.disk0 = faker.NewRandomInterval(40*time.Second,
		1200*time.Second,
		100*time.Millisecond,
		6500*time.Millisecond,
		faker.NewRandomBlinker(15, 40, 1*time.Second, 10*time.Second))

	p.disk1 = faker.NewRandomInterval(40*time.Second,
		1200*time.Second,
		100*time.Millisecond,
		6500*time.Millisecond,
		faker.NewRandomBlinker(15, 40, 1*time.Second, 10*time.Second))
}
