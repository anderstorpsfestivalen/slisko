package pixel

type Pixel struct {
	R float64
	G float64
	B float64
}

func (p *Pixel) SetColor(r float64, g float64, b float64) {
	p.R = r
	p.G = g
	p.B = b
}

func (p *Pixel) SetClamped(r float64, g float64, b float64) {
	p.R = Clamp01(r)
	p.G = Clamp01(g)
	p.B = Clamp01(b)
}

func Clamp01(v float64) float64 {
	if v < 0.0 {
		return 0.0
	}
	if v > 1.0 {
		return 1.0
	}

	return v
}
