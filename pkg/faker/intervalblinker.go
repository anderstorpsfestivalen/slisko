package faker

import (
	"math"
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
)

type IntervalBlinker struct {
	st    time.Time
	iv    time.Duration
	bl    time.Duration
	speed float64
}

func NewIntervalBlinker(interval time.Duration, blinkLength time.Duration, speed float64) *IntervalBlinker {
	return &IntervalBlinker{
		st:    time.Now(),
		iv:    interval,
		bl:    blinkLength,
		speed: speed,
	}
}

func (b *IntervalBlinker) Trig() float64 {
	if time.Since(b.st)%b.iv < b.bl {
		return utils.Square(math.Sin(b.speed * time.Since(b.st).Seconds()))
	}
	return 0
}
