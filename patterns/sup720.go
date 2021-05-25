package patterns

import (
	"math"
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/faker"
	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
)

type SUP720 struct {
	disk0 *faker.RandomInterval
	disk1 *faker.Interval
}

func (p *SUP720) Render(info RenderInfo, c *chassi.Chassi) {
	for _, port := range c.GetCardOfType("sup720") {
		sys := utils.Square(math.Sin(utils.Random(0.00001, 0.01) * time.Since(info.Start).Seconds()))
		port.Labeled["system"].SetClamped(0.0, sys, 0.0)

		port.Labeled["active"].SetClamped(0.0, 1.0, 0.0)
		port.Labeled["mgmt"].SetClamped(1.0, 0.5, 0.0)

		port.Labeled["disk0"].SetClamped(0.0, p.disk0.Trig(), 0.0)

		//disk1 := utils.Square(math.Sin(utils.Random(1, 5) * time.Since(info.Start).Seconds()))
		//disk1 := utils.Square(math.Sin(20 * time.Since(info.Start).Seconds()))
		port.Labeled["disk1"].SetClamped(0.0, p.disk1.Trig(), 0.0)

	}
}

func (p *SUP720) Info() PatternInfo {
	return PatternInfo{
		Name:     "sup720",
		Category: "misc",
	}
}

func (p *SUP720) Bootstrap(c *chassi.Chassi) {
	p.disk0 = faker.NewRandomInterval(400*time.Millisecond,
		6000*time.Millisecond,
		100*time.Millisecond,
		3500*time.Millisecond,
		faker.NewRandomBlinker(15, 40, 1*time.Second, 10*time.Second))

	p.disk1 = faker.NewInterval(2*time.Second,
		500*time.Millisecond,
		faker.NewBlinker(30),
	)
}
