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

	glcs []glc
}

type glc struct {
	Pos    pixel.Vec
	Tex    pixel.Picture
	Sprite *pixel.Sprite
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
	err := s.loadCardDefinitons(s.c.LineCards)
	if err != nil {
		panic(err)
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

		for i, _ := range s.c.LineCards {
			s.glcs[i].Sprite.Draw(win,
				pixel.IM.Moved(
					win.Bounds().Center().
						Sub(pixel.V(432, 0)).
						Add(s.glcs[i].Pos)))
		}

		win.Update()
		<-fps
	}

}

func (s *Simulator) loadCardDefinitons([]chassi.LineCard) error {
	for i, lc := range s.c.LineCards {
		newlc := glc{}
		tex, err := loadPicture("assets/images/" + lc.Image)
		if err != nil {
			return err
		}

		newlc.Tex = tex
		newlc.Sprite = pixel.NewSprite(tex, tex.Bounds())
		newlc.Pos = pixel.V(108*float64(i), 0)

		s.glcs = append(s.glcs, newlc)
	}

	return nil
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
