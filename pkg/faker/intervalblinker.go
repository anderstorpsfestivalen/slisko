package faker

import (
	"time"
)

type Interval struct {
	blink *Blinker
	st    time.Time
	iv    time.Duration
	bl    time.Duration
}

func NewInterval(interval time.Duration, blinkLength time.Duration, blink *Blinker) *Interval {
	return &Interval{
		blink: blink,
		st:    time.Now(),
		iv:    interval,
		bl:    blinkLength,
	}
}

func (b *Interval) Trig() float64 {
	if time.Since(b.st)%b.iv < b.bl {
		return b.blink.Trig()
	}
	return 0
}
