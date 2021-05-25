package faker

import (
	"math"
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
)

type SteppedBlinker struct {
	st           time.Time
	refTime      *time.Time
	steps        []float64
	currentSpeed float64
}

func NewSteppedBlinker(steps []float64, refTime *time.Time) *SteppedBlinker {
	s := SteppedBlinker{
		st:      time.Now(),
		refTime: refTime,
		steps:   steps,
	}
	s.newStep()
	return &s
}

func (b *SteppedBlinker) Trig() float64 {
	return utils.Square(math.Sin(b.currentSpeed * time.Since(b.st).Seconds()))
}

func (b SteppedBlinker) newStep() {

}
