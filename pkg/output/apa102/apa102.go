package apa102

import (
	"fmt"

	log "github.com/sirupsen/logrus"

	"github.com/anderstorpsfestivalen/slisko/pkg/pixel"
	"periph.io/x/conn/v3/physic"
	"periph.io/x/conn/v3/spi"
	"periph.io/x/conn/v3/spi/spireg"
	"periph.io/x/devices/v3/apa102"
	"periph.io/x/host/v3"
)

type APA102 struct {
	port      spi.PortCloser
	strip     *apa102.Dev
	mapping   []*pixel.Pixel
	numPixels int64

	initated bool
}

func New(port string, numPixels int64, brightness uint8, portSpeed string, trigger chan bool) (*APA102, error) {
	if _, err := host.Init(); err != nil {
		return &APA102{}, err
	}

	s1, err := spireg.Open(port)
	if err != nil {
		return &APA102{}, err
	}

	if p, ok := s1.(spi.Pins); ok {
		log.WithFields(log.Fields{
			"CLK":  p.CLK(),
			"MOSI": p.MOSI(),
		}).Info("SPI Pins")
	}

	opts := apa102.DefaultOpts
	opts.NumPixels = int(numPixels)
	opts.Intensity = brightness
	strip, err := apa102.New(s1, &opts)
	if err != nil {
		return &APA102{}, err
	}

	sp := 10 * physic.MegaHertz
	if portSpeed != "" {
		sp.Set(portSpeed)
	}
	s1.LimitSpeed(sp)

	return &APA102{
		port:      s1,
		numPixels: numPixels,
		strip:     strip,
		initated:  true,
	}, nil
}

func (a *APA102) Map(nm []pixel.Pixel) {
	for z, _ := range nm {
		a.mapping = append(a.mapping, &nm[z])
	}
}

func (a *APA102) GetMap() *[]*pixel.Pixel {
	return &a.mapping
}

func (a *APA102) Write(pixels *[]byte) (int, error) {
	if a.initated {
		return a.strip.Write(*pixels)
	}
	return 0, fmt.Errorf("apa102 not initalized")
}

func (a *APA102) Clear() {
	data := make([]byte, a.numPixels*3)
	a.Write(&data)
}

func (a *APA102) Close() error {
	return a.port.Close()
}
