package simulator

import (
	"image"
	"os"

	_ "image/png"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/gopxl/pixel/v2"
	"github.com/gopxl/pixel/v2/backends/opengl"
	"github.com/gopxl/pixel/v2/ext/imdraw"
)

type Simulator struct {
	c             chassi.Chassi
	width         int
	height        int
	fps           int
	renderTrigger chan bool

	glcs []glc
	LEDs []*imdraw.IMDraw
}

type glc struct {
	Pos    pixel.Vec
	Tex    pixel.Picture
	Sprite *pixel.Sprite
}

func New(c chassi.Chassi, width int, height int, trigger chan bool) Simulator {
	return Simulator{
		c:      c,
		width:  width,
		height: height,

		renderTrigger: trigger,
	}
}

func (s *Simulator) Start() {
	opengl.Run(s.run)
}

func (s *Simulator) run() {
	err := s.loadCardDefinitons(s.c.LineCards)
	if err != nil {
		panic(err)
	}

	width := 108 * len(s.c.LineCards)

	cfg := opengl.WindowConfig{
		Title:  "Slisko Simulator",
		Bounds: pixel.R(0, 0, float64(width), float64(s.height)),
		VSync:  true,
	}
	win, err := opengl.NewWindow(cfg)
	if err != nil {
		panic(err)
	}

	//win.SetPos(pixel.V(50, 300))

	for !win.Closed() {

		s.generateLEDs(win)

		for i, _ := range s.c.LineCards {
			s.glcs[i].Sprite.Draw(win,
				pixel.IM.Moved(
					win.Bounds().Center().
						Sub(pixel.V(float64(108*(len(s.c.LineCards)-1)/2), 0)).
						Add(s.glcs[i].Pos)))
		}
		for _, p := range s.LEDs {
			p.Draw(win)
		}

		win.Update()
		<-s.renderTrigger
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

func (s *Simulator) generateLEDs(w *opengl.Window) {

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
