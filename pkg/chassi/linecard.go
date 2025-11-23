package chassi

import (
	"fmt"
	"strconv"

	"github.com/anderstorpsfestivalen/slisko/pkg/pixel"
)

type LineCard struct {
	Name   string
	Image  string
	Active bool
	LEDs   []pixel.Pixel

	Status  *pixel.Pixel
	Link    []*pixel.Pixel
	Labeled map[string]*pixel.Pixel
}

func getSliceAddr(slice []pixel.Pixel, s int, e int) []*pixel.Pixel {

	ym := make([]*pixel.Pixel, e-s)
	k := 0
	for i := s; i < e; i++ {
		ym[k] = &slice[i]
		k++
	}
	return ym
}

func getSliceMap(m []*pixel.Pixel, prefix string) map[string]*pixel.Pixel {
	o := make(map[string]*pixel.Pixel)
	for i, v := range m {
		o[prefix+strconv.Itoa(i+1)] = v
	}
	return o
}

func setManyPixelPositons(pixels []pixel.Pixel, p []pixel.Position) {
	// Bounds check to prevent panic
	numPixels := len(pixels)
	for i, pi := range p {
		if i >= numPixels {
			// Should never happen in production, but protects against bad linecard definitions
			panic(fmt.Sprintf("setManyPixelPositons: position array has %d entries but pixel array only has %d entries (index %d out of bounds)",
				len(p), numPixels, i))
		}
		pixels[i].SetPosition(pi.X, pi.Y, pi.Size)
	}
}
