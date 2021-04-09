package simulator

import (
	"image"
	"os"
	"time"

	_ "image/png"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/faiface/pixel"
	"github.com/faiface/pixel/imdraw"
	"github.com/faiface/pixel/pixelgl"
)

type Simulator struct {
	c      chassi.Chassi
	width  int
	height int
	fps    int

	glcs []glc
	LEDs []*imdraw.IMDraw
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

		s.generateLEDs(win)

		for i, _ := range s.c.LineCards {
			s.glcs[i].Sprite.Draw(win,
				pixel.IM.Moved(
					win.Bounds().Center().
						Sub(pixel.V(432, 0)).
						Add(s.glcs[i].Pos)))
		}
		for _, p := range s.LEDs {
			p.Draw(win)
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

func (s *Simulator) generateLEDs(w *pixelgl.Window) {

	bounds := w.Bounds()
	s.LEDs = s.LEDs[:0]

	for i, lc := range s.c.LineCards {
		for _, pi := range lc.LEDs {
			pos := pi.GetPositon()
			p := imdraw.New(nil)
			p.Color = pixel.RGB(pi.R, pi.G, pi.B)
			p.Push(pixel.V(pos.X+(108*float64(i)), bounds.Max.Y-pos.Y))
			p.Circle(pos.Size, 0)

			s.LEDs = append(s.LEDs, p)
		}
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
