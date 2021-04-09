package chassi

import (
	"github.com/anderstorpsfestivalen/slisko/pkg/pixel"
)

type LineCard struct {
	Name  string
	Image string
	LEDs  []pixel.Pixel

	Status *pixel.Pixel
	Link   []*pixel.Pixel
	Misc   map[string]*pixel.Pixel
}

func getSliceAddr(slice []pixel.Pixel, s int, e int) []*pixel.Pixel {

	ym := make([]*pixel.Pixel, e-s)
	k := 0
	for i := s; i < e; i++ {
		ym[k] = &slice[i-1]
		k++
	}
	return ym
}
