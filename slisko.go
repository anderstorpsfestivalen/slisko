package main

import (
	"flag"
	"os"
	"os/signal"
	"syscall"

	"github.com/anderstorpsfestivalen/slisko/pkg/api"
	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/configuration"
	"github.com/anderstorpsfestivalen/slisko/pkg/console"
	"github.com/anderstorpsfestivalen/slisko/pkg/controller"
	"github.com/anderstorpsfestivalen/slisko/pkg/gpio"
	"github.com/anderstorpsfestivalen/slisko/pkg/output"
	"github.com/anderstorpsfestivalen/slisko/pkg/output/apa102"
	"github.com/anderstorpsfestivalen/slisko/pkg/output/null"
	"github.com/anderstorpsfestivalen/slisko/pkg/output/wledapa"
	"github.com/anderstorpsfestivalen/slisko/simulator"
	"github.com/coral/ddp"
	log "github.com/sirupsen/logrus"
)

func main() {
	flag.Bool("simulator", false, "enables the simulator")
	flag.Bool("console", false, "Enables LED console op")
	flag.Bool("spi", false, "Enables LED spi op")
	flag.Bool("ddp", false, "Enables DDP output")
	flag.Bool("gpio", false, "Enables DDP output")
	flag.Bool("print-config", false, "print the loaded configuration and exit")
	configurationFile := flag.String("config", "configurations/9010.toml", "configuration file")
	ddpHost := flag.String("ddphost", "", "ddp host")
	apiAddr := flag.String("api", "0.0.0.0:3000", "address for API server to listen on")
	brightness := flag.Uint("brightness", 255, "override global brightness")
	fps := flag.Int("fps", 60, "override the FPS")
	numLeds := flag.Int64("leds", 132, "number of leds")
	flag.Parse()

	log.Info("Started Slisko Controller")

	def, err := configuration.LoadFromFile(*configurationFile)
	if err != nil {
		log.Error(err)
		panic(err)
	}

	// Print configuration and exit if requested
	if isFlagPassed("print-config") {
		def.PrintWithSource(*configurationFile)
		os.Exit(0)
	}

	*numLeds = def.LEDAmount

	c := chassi.New(chassi.CardsFromDefinition(def.Linecards))

	log.WithFields(log.Fields{
		"linecards": c.GetCardOrder(),
		"#":         len(c.LineCards),
	}).Info("Created a new chassi")

	ctrl := controller.New(&c)
	ctrl.Start(*fps)

	for _, p := range def.Patterns {
		ctrl.EnablePattern(p)
	}

	// ... (after flag.Parse())
	// api := api.New(&c, &ctrl)
	// go api.Start("0.0.0.0:3000")
	api := api.New(&c, &ctrl)
	go api.Start(*apiAddr)

	if isFlagPassed("gpio") {
		if def.UsesButtons() {
			_, err := gpio.NewGPIOController(def.Buttons, &ctrl)
			if err != nil {
				log.Error(err)
				panic(err)
			}
		}
	}

	var selectedDevice output.Device
	// Null device to run program without outputs
	selectedDevice = &null.Null{}

	// check if direct spi is enabled and setup
	if isFlagPassed("spi") {
		sl, err := apa102.New("/dev/spidev0.0",
			*numLeds,           // NUM LEDS
			uint8(*brightness), //BRIGHTNESS
			"20Mhz",            // MHZ (not used rihgt now hahahaha)
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
				sl.Clear()
				sl.Close()
				os.Exit(1)
			}()
		}
		selectedDevice = sl
	}

	if isFlagPassed("ddp") {
		ddpClient := ddp.NewDDPController()

		err := ddpClient.ConnectUDP(*ddpHost)
		if err != nil {
			panic(err)
		}

		ww := wledapa.NewWLEDAPA(ddpClient)

		selectedDevice = ww

	}

	op, err := output.New(*numLeds, selectedDevice, ctrl.FrameBroker.Subscribe())
	if err != nil {
		log.Error(err)
		panic(err)
	}

	for i, m := range def.Mapping {
		if m.IsGen() {
			op.Map(output.GenEmpty(*m.Gen))
		}
		if m.IsCard() {
			cardIdx := *m.Card
			if cardIdx < 0 || cardIdx >= len(c.LineCards) {
				log.WithFields(log.Fields{
					"mapping_index": i,
					"card_index":    cardIdx,
					"num_linecards": len(c.LineCards),
				}).Error("Card index out of bounds in mapping")
				panic("Card index out of bounds - this should have been caught during config validation")
			}
			op.Map(c.LineCards[cardIdx].LEDs)
		}
	}

	// Start output pump
	go op.Run()

	if isFlagPassed("console") {
		console := console.New(op.GetMap(), ctrl.FrameBroker.Subscribe())
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
