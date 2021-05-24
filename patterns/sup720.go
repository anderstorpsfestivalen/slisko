package patterns

import (
	"math"
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/faker"
	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
)

type SUP720 struct {
	disk1 *faker.IntervalBlinker
}

func (p *SUP720) Render(info RenderInfo, c *chassi.Chassi) {
	for _, port := range c.GetCardOfType("sup720") {
		sys := utils.Square(math.Sin(utils.Random(0.00001, 0.01) * time.Since(info.Start).Seconds()))
		port.Labeled["system"].SetClamped(0.0, sys, 0.0)

		port.Labeled["active"].SetClamped(0.0, 1.0, 0.0)
		port.Labeled["mgmt"].SetClamped(1.0, 0.5, 0.0)

		disk0 := utils.Square(math.Sin(utils.Random(1, 5) * time.Since(info.Start).Seconds()))
		port.Labeled["disk0"].SetClamped(0.0, disk0, 0.0)

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

func (p *SUP720) Bootstrap() {
	p.disk1 = faker.NewIntervalBlinker(2*time.Second, 500*time.Millisecond, 30)
}
