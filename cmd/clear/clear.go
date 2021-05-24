package main

import (
	"flag"

	log "github.com/sirupsen/logrus"

	"periph.io/x/conn/v3/spi"
	"periph.io/x/conn/v3/spi/spireg"
	"periph.io/x/devices/v3/apa102"
	"periph.io/x/host/v3"
)

func main() {

	numPixels := flag.Int("pixels", 144, "numpixels")
	flag.Parse()

	_, err := host.Init()
	if err != nil {
		panic(err)
	}
	s1, err := spireg.Open("/dev/spidev0.0")
	if err != nil {
		panic(err)
	}

	defer s1.Close()
	if p, ok := s1.(spi.Pins); ok {
		log.WithFields(log.Fields{
			"CLK":  p.CLK(),
			"MOSI": p.MOSI(),
		}).Info("SPI Pins")
	}
	opts := apa102.DefaultOpts
	opts.NumPixels = int(*numPixels)
	strip, err := apa102.New(s1, &opts)

	out := make([]byte, *numPixels*3)

	strip.Write(out)
}
