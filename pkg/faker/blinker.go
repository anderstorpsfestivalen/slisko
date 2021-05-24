package faker

import (
	"math"
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
)

type Blinker struct {
	st    time.Time
	speed float64
}

func NewBlinker(speed float64) *Blinker {
	return &Blinker{
		st:    time.Now(),
		speed: speed,
	}
}

func (b *Blinker) Trig() float64 {
	return utils.Square(math.Sin(b.speed * time.Since(b.st).Seconds()))
}
