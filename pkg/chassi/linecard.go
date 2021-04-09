package chassi

import (
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
	for i, pi := range p {
		pixels[i].SetPosition(pi.X, pi.Y, pi.Size)
	}
}
