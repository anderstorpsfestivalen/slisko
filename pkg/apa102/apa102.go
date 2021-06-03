package apa102

import (
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

	outputBuf []byte

	initated bool
}

func New(port string, numPixels int64, brightness uint8, portSpeed string, trigger chan bool) (*APA102, error) {
	if _, err := host.Init(); err != nil {
		return &APA102{renderTrigger: trigger, outputBuf: make([]byte, numPixels*3)}, err
	}

	s1, err := spireg.Open(port)
	if err != nil {
		return &APA102{renderTrigger: trigger, outputBuf: make([]byte, numPixels*3)}, err
	}

	if p, ok := s1.(spi.Pins); ok {
		log.WithFields(log.Fields{
			"CLK":  p.CLK(),
			"MOSI": p.MOSI(),
		}).Info("SPI Pins")
	}

	sp := 20 * physic.MegaHertz
	if portSpeed != "" {
		sp.Set(portSpeed)
	}
	s1.LimitSpeed(sp)

	opts := apa102.DefaultOpts
	opts.NumPixels = int(numPixels)
	opts.Intensity = brightness
	strip, err := apa102.New(s1, &opts)
	if err != nil {
		return &APA102{renderTrigger: trigger, outputBuf: make([]byte, numPixels*3)}, err
	}

	return &APA102{
		port:          s1,
		strip:         strip,
		renderTrigger: trigger,
		outputBuf:     make([]byte, numPixels*3),
		initated:      true,
	}, nil
}

func (a *APA102) Run() {
	for {
		<-a.renderTrigger
		for i, l := range a.mapping {
			a.outputBuf[i*3] = pixel.Clamp255(l.R * 255)
			a.outputBuf[i*3+1] = pixel.Clamp255(l.G * 255)
			a.outputBuf[i*3+2] = pixel.Clamp255(l.B * 255)
		}

		if a.initated {
			a.strip.Write(a.outputBuf)
		}
	}
}

func (a *APA102) Map(nm []pixel.Pixel) {
	for z, _ := range nm {
		a.mapping = append(a.mapping, &nm[z])
	}
}

func (a *APA102) GetMap() *[]*pixel.Pixel {
	return &a.mapping
}

func GenEmpty(num int) []pixel.Pixel {
	lp := []pixel.Pixel{}

	for i := 0; i < num; i++ {
		lp = append(lp, pixel.Pixel{})
	}
	return lp
}

func (a *APA102) Clear() {
	for i, _ := range a.outputBuf {
		a.outputBuf[i] = 0
	}
	if a.initated {
		a.strip.Write(a.outputBuf)
	}
}

func (a *APA102) Close() {
	a.port.Close()
}
