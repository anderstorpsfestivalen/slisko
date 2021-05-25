package main

import (
	"flag"
	"os"
	"os/signal"
	"syscall"

	"github.com/anderstorpsfestivalen/slisko/pkg/apa102"
	"github.com/anderstorpsfestivalen/slisko/pkg/api"
	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/console"
	"github.com/anderstorpsfestivalen/slisko/pkg/controller"
	"github.com/anderstorpsfestivalen/slisko/simulator"
	log "github.com/sirupsen/logrus"
)

func main() {
	flag.Bool("simulator", false, "enables the simulator")
	flag.Bool("console", false, "Enables LED console output")
	brightness := flag.Uint("brightness", 255, "override global brightness")
	fps := flag.Int("fps", 60, "override the FPS")
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

	ctrl := controller.New(&c)
	ctrl.Start(*fps)
	ctrl.EnablePattern("blink48ports")
	ctrl.EnablePattern("greenstatus")
	ctrl.EnablePattern("sup720")
	ctrl.EnablePattern("x6704")

	api := api.New(&c, &ctrl)
	go api.Start("0.0.0.0:3000")

	//APA102 DEFINITION

	apa, err := apa102.New("/dev/spidev0.0",
		144,                // NUM LEDS
		uint8(*brightness), //BRIGHTNESS
		8,                  // MHZ (not used rihgt now hahahaha)
		ctrl.FrameBroker.Subscribe())
	if err != nil {
		log.Error(err)
		log.Error("SPI FAILED TO INITALIZE, THE LED STRIP WILL NOT WORK")
	} else {
		//Clear strip when exiting
		c := make(chan os.Signal, 1)
		signal.Notify(c, os.Interrupt, syscall.SIGTERM)
		go func() {
			<-c
			ctrl.Stop()
			apa.Clear()
			apa.Close()
			os.Exit(1)
		}()
	}

	//Generate a test 6704 + 1 blank
	apa.Map(c.LineCards[3].LEDs)
	apa.Map(apa102.GenEmpty(1))
	go apa.Run()

	if isFlagPassed("console") {
		console := console.New(apa.GetMap(), ctrl.FrameBroker.Subscribe())
		go console.Run()
	}

	if isFlagPassed("simulator") {
		sim := simulator.New(c, (108 * 9), 1000, ctrl.FrameBroker.Subscribe())
		sim.Start()
	} else {
		select {}
	}

}

func isFlagPassed(name string) bool {
	found := false
	flag.Visit(func(f *flag.Flag) {
		if f.Name == name {
			found = true
		}
	})
	return found
}
