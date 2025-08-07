package output

import (
	"bytes"

	"github.com/anderstorpsfestivalen/slisko/pkg/pixel"
)

type Device interface {
	Write(pixels []byte) (int, error)
	Close() error
}

type Output struct {
	mapping       []*pixel.Pixel
	renderTrigger chan bool

	outputBuf []byte
	lastBuf   []byte

	device     Device
	frameCount int64
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

		a.frameCount++

		if !bytes.Equal(a.outputBuf, a.lastBuf) || a.frameCount >= 60 {

			a.device.Write(a.outputBuf)

			a.lastBuf = append([]byte(nil), a.outputBuf...)

			if a.frameCount >= 60 {
				a.frameCount = 0
			}
		}
	}
}

func (a *Output) Map(nm []pixel.Pixel) {
	for z := range nm {
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
	for i := range a.outputBuf {
		a.outputBuf[i] = 0
	}
	a.device.Write(a.outputBuf)
}

func (a *Output) Close() {
	a.device.Close()
}
