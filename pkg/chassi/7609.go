package chassi

import "github.com/anderstorpsfestivalen/slisko/pkg/pixel"

func Gen7609Chassi() []LineCard {

	chassi := []LineCard{
		Gen6478(),
		Gen6704(),
		GenBlank(),
		Gen6704(),
		GenSUP720(),
		Gen6704(),
		GenBlank(),
		Gen6704(),
		Gen6478(),
	}

	return chassi
}

func Gen6478() LineCard {

	leds := make([]pixel.Pixel, 49)

	l := LineCard{
		Name:   "6478",
		Image:  "6478.png",
		Active: true,
		LEDs:   leds,

		Status: &leds[0],
		Link:   getSliceAddr(leds, 1, 49),
		Labeled: map[string]*pixel.Pixel{
			"status": &leds[0],
		},
	}

	for k, v := range getSliceMap(getSliceAddr(leds, 1, 49), "p") {
		l.Labeled[k] = v
	}

	setManyPixelPositons(l.LEDs, []pixel.Position{
		{X: 11, Y: 73, Size: 5},

		{X: 11, Y: 88, Size: 5},
		{X: 11, Y: 101, Size: 5},
		{X: 11, Y: 114, Size: 5},
		{X: 11, Y: 128, Size: 5},
		{X: 11, Y: 141, Size: 5},
		{X: 11, Y: 154, Size: 5},
		{X: 11, Y: 167, Size: 5},
		{X: 11, Y: 180, Size: 5},
		{X: 11, Y: 194, Size: 5},
		{X: 11, Y: 207, Size: 5},
		{X: 11, Y: 220, Size: 5},
		{X: 11, Y: 233, Size: 5},

		{X: 11, Y: 323, Size: 5},
		{X: 11, Y: 336, Size: 5},
		{X: 11, Y: 349, Size: 5},
		{X: 11, Y: 362, Size: 5},
		{X: 11, Y: 375, Size: 5},
		{X: 11, Y: 387, Size: 5},
		{X: 11, Y: 400, Size: 5},
		{X: 11, Y: 413, Size: 5},
		{X: 11, Y: 426, Size: 5},
		{X: 11, Y: 439, Size: 5},
		{X: 11, Y: 451, Size: 5},
		{X: 11, Y: 464, Size: 5},

		{X: 11, Y: 549, Size: 5},
		{X: 11, Y: 562, Size: 5},
		{X: 11, Y: 575, Size: 5},
		{X: 11, Y: 588, Size: 5},
		{X: 11, Y: 601, Size: 5},
		{X: 11, Y: 613, Size: 5},
		{X: 11, Y: 626, Size: 5},
		{X: 11, Y: 639, Size: 5},
		{X: 11, Y: 652, Size: 5},
		{X: 11, Y: 665, Size: 5},
		{X: 11, Y: 677, Size: 5},
		{X: 11, Y: 690, Size: 5},

		{X: 11, Y: 778, Size: 5},
		{X: 11, Y: 791, Size: 5},
		{X: 11, Y: 804, Size: 5},
		{X: 11, Y: 817, Size: 5},
		{X: 11, Y: 830, Size: 5},
		{X: 11, Y: 842, Size: 5},
		{X: 11, Y: 855, Size: 5},
		{X: 11, Y: 868, Size: 5},
		{X: 11, Y: 881, Size: 5},
		{X: 11, Y: 894, Size: 5},
		{X: 11, Y: 906, Size: 5},
		{X: 11, Y: 919, Size: 5},
	})
	return l
}

func Gen6704() LineCard {

	leds := make([]pixel.Pixel, 5)

	l := LineCard{
		Name:   "6704",
		Image:  "6704.png",
		Active: true,
		LEDs:   leds,

		Status: &leds[0],
		Link:   getSliceAddr(leds, 1, 5),
		Labeled: map[string]*pixel.Pixel{
			"status": &leds[0],
		},
	}

	for k, v := range getSliceMap(getSliceAddr(leds, 1, 5), "p") {
		l.Labeled[k] = v
	}

	setManyPixelPositons(l.LEDs, []pixel.Position{
		{X: 27, Y: 55, Size: 8},

		{X: 12, Y: 110, Size: 5},
		{X: 12, Y: 129, Size: 5},
		{X: 12, Y: 148, Size: 5},
		{X: 12, Y: 168, Size: 5},
	})

	return l
}

func GenSUP720() LineCard {

	leds := make([]pixel.Pixel, 9)

	l := LineCard{
		Name:   "sup720",
		Image:  "sup720.png",
		Active: true,
		LEDs:   leds,

		Status: &leds[0],
		Link:   getSliceAddr(leds, 6, 9),
		Labeled: map[string]*pixel.Pixel{
			"status": &leds[0],
			"system": &leds[1],
			"active": &leds[2],
			"mgmt":   &leds[3],
			"disk0":  &leds[4],
			"disk1":  &leds[5],
		},
	}

	for k, v := range getSliceMap(getSliceAddr(leds, 6, 9), "p") {
		l.Labeled[k] = v
	}

	setManyPixelPositons(l.LEDs, []pixel.Position{
		{X: 24, Y: 57, Size: 5},

		{X: 24, Y: 71, Size: 5},
		{X: 24, Y: 85, Size: 5},
		{X: 24, Y: 98, Size: 5},

		{X: 54, Y: 105, Size: 5},
		{X: 28, Y: 291, Size: 5},

		{X: 31, Y: 579, Size: 5},
		{X: 31, Y: 652, Size: 5},
		{X: 32, Y: 725, Size: 5},
	})

	return l
}

func GenBlank() LineCard {

	return LineCard{
		Name:   "blank",
		Image:  "blank.png",
		Active: false,
	}
}
