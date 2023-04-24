package output

import (
	"bytes"

	"github.com/anderstorpsfestivalen/slisko/pkg/pixel"
)

type Device interface {
	Write(pixels *[]byte) (int, error)
	Close() error
}

type Output struct {
	mapping       []*pixel.Pixel
	renderTrigger chan bool

	outputBuf []byte
	lastBuf   []byte

	device Device

	initated bool
}

func New(numPixels int64, device Device, trigger chan bool) (*Output, error) {

	return &Output{
		renderTrigger: trigger,
		device:        device,
		lastBuf:       make([]byte, numPixels*3),
		outputBuf:     make([]byte, numPixels*3),
	}, nil
}

func (a *Output) Run() {
	for {
		<-a.renderTrigger
		for i, l := range a.mapping {
			a.outputBuf[i*3] = pixel.Clamp255(l.R * 255)
			a.outputBuf[i*3+1] = pixel.Clamp255(l.G * 255)
			a.outputBuf[i*3+2] = pixel.Clamp255(l.B * 255)
		}

		if !bytes.Equal(a.outputBuf, a.lastBuf) {

			if a.initated {
				a.device.Write(&a.outputBuf)
			}

			a.lastBuf = append([]byte(nil), a.outputBuf...)
		}

	}
}

func (a *Output) Map(nm []pixel.Pixel) {
	for z, _ := range nm {
		a.mapping = append(a.mapping, &nm[z])
	}
}

func (a *Output) GetMap() *[]*pixel.Pixel {
	return &a.mapping
}

func GenEmpty(num int) []pixel.Pixel {
	lp := []pixel.Pixel{}

	for i := 0; i < num; i++ {
		lp = append(lp, pixel.Pixel{})
	}
	return lp
}

func (a *Output) Clear() {
	for i, _ := range a.outputBuf {
		a.outputBuf[i] = 0
	}
	a.device.Write(&a.outputBuf)
}

func (a *Output) Close() {
	a.device.Close()
}
