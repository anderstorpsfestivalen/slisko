package main

import (
	"time"

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

	sim := simulator.New(c, (108 * 9), 1000, 60)
	sim.Start()

}

func painter(c chassi.Chassi) {
	ticker := time.NewTicker((1000 / 60) * time.Millisecond)
	go func() {
		m := 0.0
		for {
			_ = <-ticker.C
			m = m + 0.01
			for _, p := range c.StatusLEDs {
				p.SetClamped(0.1, 1.0, 0.0)
			}
			for _, p := range c.LinkPorts {
				p.SetClamped(m, m, 0.3)
			}
			if m > 1 {
				m = 0
			}

		}
	}()
}
