package console

import (
	"image"
	"image/color"
	"log"

	"github.com/anderstorpsfestivalen/slisko/pkg/pixel"
	"periph.io/x/devices/v3/screen1d"
)

type Console struct {
	mapping       *[]*pixel.Pixel
	renderTrigger chan bool
	screen        *screen1d.Dev
	image         *image.NRGBA
}

func New(mapping *[]*pixel.Pixel, trigger chan bool) *Console {
	opts := &screen1d.Opts{
		X: len(*mapping),
	}
	scr := screen1d.New(opts)
	return &Console{
		mapping:       mapping,
		renderTrigger: trigger,
		screen:        scr,
		image:         image.NewNRGBA(scr.Bounds()),
	}
}

func (c *Console) Run() {
	for {
		<-c.renderTrigger
		for i, d := range *c.mapping {
			c.image.SetNRGBA(i, 0, color.NRGBA{
				R: pixel.Clamp255(d.R * 255),
				G: pixel.Clamp255(d.G * 255),
				B: pixel.Clamp255(d.B * 255),
				A: 255,
			})
		}

		if err := c.screen.Draw(c.screen.Bounds(), c.image, image.Point{}); err != nil {
			log.Fatal(err)
		}
	}
}
