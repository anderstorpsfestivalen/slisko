package portlike

import (
	"math"
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
)

type Blinker struct {
	st time.Time
}

func NewBlinker() *Blinker {
	return &Blinker{
		st: time.Now(),
	}
}

func (b *Blinker) Trig() float64 {
	return utils.Square(math.Sin(10 * time.Since(b.st).Seconds()))
}
