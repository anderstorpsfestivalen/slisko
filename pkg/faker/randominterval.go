package faker

import (
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
)

type RandomInterval struct {
	blink Fake
	st    time.Time
	iv    time.Duration
	bl    time.Duration

	MinInterval time.Duration
	MaxInterval time.Duration
	MinBlink    time.Duration
	MaxBlink    time.Duration
}

func NewRandomInterval(minI time.Duration, maxI time.Duration, minB time.Duration, maxB time.Duration, blink Fake) *RandomInterval {
	utils.Random(float64(minI), float64(maxI))
	return &RandomInterval{
		blink: blink,
		st:    time.Now(),

		//iv: time.Duration(utils.Random(float64(minI.Milliseconds()), float64(maxI.Milliseconds())) * float64(time.Millisecond)),
		iv: time.Duration(utils.RandomInt64(minI.Milliseconds(), maxI.Milliseconds()) * int64(time.Millisecond)),
		bl: time.Duration(utils.RandomInt64(minB.Milliseconds(), maxB.Milliseconds()) * int64(time.Millisecond)),

		MinInterval: minI,
		MaxInterval: maxI,
		MinBlink:    minB,
		MaxBlink:    maxB,
	}
}

func (ri *RandomInterval) Trig() float64 {
	if time.Since(ri.st) > (ri.iv + ri.bl) {
		ri.iv = time.Duration(utils.RandomInt64(ri.MinInterval.Milliseconds(), ri.MaxInterval.Milliseconds()) * int64(time.Millisecond))
		ri.bl = time.Duration(utils.RandomInt64(ri.MinBlink.Milliseconds(), ri.MaxBlink.Milliseconds()) * int64(time.Millisecond))
		ri.st = time.Now()
	}
	if time.Since(ri.st) > ri.iv {
		return ri.blink.Trig()
	}

	return 0
}
