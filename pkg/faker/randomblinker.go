package faker

import (
	"math"
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
)

type RandomBlinker struct {
	st    time.Time
	iv    time.Duration
	speed float64

	MinSpeed float64
	MaxSpeed float64
	MinTime  time.Duration
	MaxTime  time.Duration
}

func NewRandomBlinker(minSpeed float64, maxSpeed float64, minTime time.Duration, maxTime time.Duration) *RandomBlinker {

	return &RandomBlinker{
		st: time.Now(),
		iv: time.Duration(
			utils.Random(
				float64(minTime.Milliseconds()),
				float64(maxTime.Milliseconds()),
			) * float64(time.Millisecond),
		),

		speed: utils.Random(minSpeed, maxSpeed),

		MinSpeed: minSpeed,
		MaxSpeed: maxSpeed,
		MinTime:  minTime,
		MaxTime:  maxTime,
	}
}

func (b *RandomBlinker) Trig() float64 {
	if time.Since(b.st) > b.iv {
		b.iv = b.genDur()
		b.speed = utils.Random(b.MinSpeed, b.MaxSpeed)
		b.st = time.Now()
	}

	return utils.Square(math.Sin(b.speed * time.Since(b.st).Seconds()))
}

func (b *RandomBlinker) genDur() time.Duration {
	return time.Duration(
		utils.Random(
			float64(b.MinTime.Milliseconds()),
			float64(b.MaxTime.Milliseconds()),
		) * float64(time.Millisecond),
	)
}
