package main

import (
	"flag"

	"github.com/anderstorpsfestivalen/slisko/pkg/api"
	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/controller"
	"github.com/anderstorpsfestivalen/slisko/simulator"
	log "github.com/sirupsen/logrus"
)

func main() {
	flag.Bool("simulator", false, "enables the simulator")
	flag.Parse()

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

	// output := apa102.New(&c, 60)-
	// output.Start()

	ctrl := controller.New(&c)
	ctrl.Start(60)
	ctrl.EnablePattern("blinkports")
	ctrl.EnablePattern("greenstatus")
	ctrl.EnablePattern("sup720")

	api := api.New(&c, &ctrl)
	go api.Start("0.0.0.0:3000")

	if isFlagPassed("simulator") {
		sim := simulator.New(c, (108 * 9), 1000, 60)
		sim.Start()
	} else {
		select {}
	}

}

// func painter(c chassi.Chassi) {
// 	start := time.Now()
// 	ticker := time.NewTicker((1000 / 60) * time.Millisecond)
// 	go func() {
// 		m := 0.0

// 		//	i := len(c.LineCards[0].Link)

// 		for {
// 			_ = <-ticker.C
// 			m = m + 0.01
// 			_ = math.Sin(200 * time.Since(start).Seconds())

// 			for _, c := range c.GetCardOfType("sup720") {
// 				c.Labeled["disk0"].SetClamped(1.0, 0.2, 0.0)
// 			}

// 			for _, l := range c.GetLEDsWithLabel("mgmt") {
// 				l.SetClamped(1.0, 0.4, 0.7)
// 			}

// 			for _, p := range c.StatusLEDs {
// 				p.SetClamped(1.0, 0.0, 0.0)
// 			}

// 			for _, p := range c.LinkPorts {
// 				p.SetClamped(m, m, 0.3)
// 			}

// 			for _, p := range c.GetLEDsWithLabel("p3") {
// 				p.SetClamped(0.0, 0.0, 1.0)
// 			}

// 			// if i <= 0 {
// 			// 	i = len(c.LineCards[0].Link)
// 			// } else {
// 			// 	i--
// 			// }
// 			// for k := len(c.LineCards[0].Link); i < k; k-- {
// 			// 	c.LineCards[0].Link[k-1].SetClamped(1.0, 1.0, 0.0)
// 			// }
// 			// for k := 0; k < len(c.LineCards[0].Link); k++ {
// 			// 	if i > k {
// 			// 		c.LineCards[0].Link[k].SetClamped(1.0, 1.0, 0.0)
// 			// 	} else {
// 			// 		c.LineCards[0].Link[k].SetClamped(0.0, 0.0, 0.0)
// 			// 	}
// 			// }

// 			if m > 1 {
// 				m = 0
// 			}

// 		}
// 	}()
// }

func isFlagPassed(name string) bool {
	found := false
	flag.Visit(func(f *flag.Flag) {
		if f.Name == name {
			found = true
		}
	})
	return found
}
