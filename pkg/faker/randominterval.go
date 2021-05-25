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

	MinInterval int64
	MaxInterval int64
	MinBlink    int64
	MaxBlink    int64
}

func NewRandomInterval(minI int64, maxI int64, minB int64, maxB int64, blink Fake) *RandomInterval {

	utils.Random(float64(minI), float64(maxI))
	return &RandomInterval{
		blink: blink,
		st:    time.Now(),

		iv: time.Duration(utils.Random(float64(minI), float64(maxI)) * float64(time.Millisecond)),
		bl: time.Duration(utils.Random(float64(minB), float64(maxB)) * float64(time.Millisecond)),

		MinInterval: minI,
		MaxInterval: maxI,
		MinBlink:    minB,
		MaxBlink:    maxB,
	}
}

func (ri *RandomInterval) Trig() float64 {
	if time.Since(ri.st) > (ri.iv + ri.bl) {
		ri.iv = time.Duration(utils.Random(float64(ri.MaxInterval), float64(ri.MaxInterval)) * float64(time.Millisecond))
		ri.bl = time.Duration(utils.Random(float64(ri.MinBlink), float64(ri.MaxBlink)) * float64(time.Millisecond))
		ri.st = time.Now()
	}

	if time.Since(ri.st) > ri.iv {
		return ri.blink.Trig()
	}

	return 0
}
