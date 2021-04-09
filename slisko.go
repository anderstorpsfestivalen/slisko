package main

import (
	"math"
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/apa102"
	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/simulator"
	log "github.com/sirupsen/logrus"
)

func main() {
	log.Info("Started Slisko Controller")

	c := chassi.New([]chassi.LineCard{
		chassi.Gen6478(),
		chassi.Gen6704(),
		chassi.GenBlank(),
		chassi.Gen6704(),
		chassi.GenSUP720(),
		chassi.Gen6704(),
		chassi.GenBlank(),
		chassi.Gen6704(),
		chassi.Gen6478(),
	})

	log.WithFields(log.Fields{
		"linecards": c.GetCardOrder(),
		"#":         len(c.LineCards),
	}).Info("Created a new chassi")

	painter(c)

	output := apa102.New(&c, 60)
	output.Start()

	sim := simulator.New(c, (108 * 9), 1000, 60)
	sim.Start()

}

func painter(c chassi.Chassi) {
	start := time.Now()
	ticker := time.NewTicker((1000 / 60) * time.Millisecond)
	go func() {
		m := 0.0

		//	i := len(c.LineCards[0].Link)

		for {
			_ = <-ticker.C
			m = m + 0.01
			_ = math.Sin(200 * time.Since(start).Seconds())

			for _, c := range c.GetCardOfType("sup720") {
				c.Labeled["disk0"].SetClamped(1.0, 0.2, 0.0)
			}

			for _, l := range c.GetLEDsWithLabel("mgmt") {
				l.SetClamped(1.0, 0.4, 0.7)
			}

			for _, p := range c.StatusLEDs {
				p.SetClamped(1.0, 0.0, 0.0)
			}

			for _, p := range c.LinkPorts {
				p.SetClamped(m, m, 0.3)
			}

			for _, p := range c.GetLEDsWithLabel("p3") {
				p.SetClamped(0.0, 0.0, 1.0)
			}

			// if i <= 0 {
			// 	i = len(c.LineCards[0].Link)
			// } else {
			// 	i--
			// }
			// for k := len(c.LineCards[0].Link); i < k; k-- {
			// 	c.LineCards[0].Link[k-1].SetClamped(1.0, 1.0, 0.0)
			// }
			// for k := 0; k < len(c.LineCards[0].Link); k++ {
			// 	if i > k {
			// 		c.LineCards[0].Link[k].SetClamped(1.0, 1.0, 0.0)
			// 	} else {
			// 		c.LineCards[0].Link[k].SetClamped(0.0, 0.0, 0.0)
			// 	}
			// }

			if m > 1 {
				m = 0
			}

		}
	}()
}
