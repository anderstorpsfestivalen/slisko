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
	//t := time.Now()
	ticker := time.NewTicker((1000 / 60) * time.Millisecond)
	go func() {
		for {
			_ = <-ticker.C

			//t := time.Since(t)

			for _, p := range c.LinkPorts {
				//m := 1.0
				p.B = 1.0
			}

		}
	}()
}
