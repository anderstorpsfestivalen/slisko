package apa102

import (
	"strconv"

	log "github.com/sirupsen/logrus"

	"github.com/anderstorpsfestivalen/slisko/pkg/pixel"
	"periph.io/x/conn/v3/physic"
	"periph.io/x/conn/v3/spi"
	"periph.io/x/conn/v3/spi/spireg"
	"periph.io/x/devices/v3/apa102"
	"periph.io/x/host/v3"
)

type APA102 struct {
	port          spi.PortCloser
	strip         *apa102.Dev
	mapping       []*pixel.Pixel
	renderTrigger chan bool
}

func New(port string, numPixels int64, mhz int64, trigger chan bool) (*APA102, error) {
	if _, err := host.Init(); err != nil {
		return nil, err
	}

	s1, err := spireg.Open(port)
	if err != nil {
		return nil, err
	}

	dd := physic.MegaHertz
	dd.Set(strconv.FormatInt(mhz, 10) + "MHz")
	s1.LimitSpeed(dd)
	if p, ok := s1.(spi.Pins); ok {
		log.WithFields(log.Fields{
			"CLK":  p.CLK(),
			"MOSI": p.MOSI(),
		}).Info("SPI Pins")
	}

	opts := apa102.DefaultOpts
	opts.NumPixels = int(numPixels)
	strip, err := apa102.New(s1, &opts)
	if err != nil {
		return nil, err
	}

	return &APA102{
		port:          s1,
		strip:         strip,
		renderTrigger: trigger,
	}, nil
}

func (a *APA102) Run() {
	for {
		<-a.renderTrigger
		var op = []byte{}
		for _, l := range a.mapping {
			op = append(op, []byte{
				pixel.Clamp255(l.R * 255),
				pixel.Clamp255(l.G * 255),
				pixel.Clamp255(l.B * 255),
			}...)
		}
	}
}

func (a *APA102) Map(nm []pixel.Pixel) {
	for z, _ := range nm {
		a.mapping = append(a.mapping, &nm[z])
	}
}

func GenEmpty(num int) []pixel.Pixel {
	lp := []pixel.Pixel{}

	for i := 0; i < num; i++ {
		lp = append(lp, pixel.Pixel{})
	}
	return lp
}
