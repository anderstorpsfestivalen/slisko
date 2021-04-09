package simulator

import (
	"image"
	"os"
	"time"

	_ "image/png"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/faiface/pixel"
	"github.com/faiface/pixel/pixelgl"
)

type Simulator struct {
	c      chassi.Chassi
	width  int
	height int
	fps    int

	tex []pixel.Picture
}

func New(c chassi.Chassi, width int, height int, fps int) Simulator {
	return Simulator{
		c:      c,
		width:  width,
		height: height,
		fps:    fps,
	}
}

func (s *Simulator) Start() {
	pixelgl.Run(s.run)
}

func (s *Simulator) run() {
	for _, lc := range s.c.LineCards {
		tex, err := loadPicture("assets/images/" + lc.Image)
		if err != nil {
			panic(err)
		}
		s.tex = append(s.tex, tex)
	}

	cfg := pixelgl.WindowConfig{
		Title:  "Slisko Simulator",
		Bounds: pixel.R(0, 0, float64(s.width), float64(s.height)),
		VSync:  true,
	}
	win, err := pixelgl.NewWindow(cfg)
	if err != nil {
		panic(err)
	}

	fps := time.Tick(time.Second / time.Duration(s.fps))

	for !win.Closed() {

		win.Update()
		<-fps
	}

}

func loadPicture(path string) (pixel.Picture, error) {
	file, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer file.Close()
	img, _, err := image.Decode(file)
	if err != nil {
		return nil, err
	}
	return pixel.PictureDataFromImage(img), nil
}
