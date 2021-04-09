package chassi

import "github.com/anderstorpsfestivalen/slisko/pkg/pixel"

func Gen6478() LineCard {

	leds := make([]pixel.Pixel, 49)

	l := LineCard{
		Name:   "6478",
		Image:  "6478.png",
		Active: true,
		LEDs:   leds,

		Status: &leds[0],
		Link:   getSliceAddr(leds, 1, 49),
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
	})
	return l
}

func Gen6704() LineCard {

	leds := make([]pixel.Pixel, 5)

	return LineCard{
		Name:   "6704",
		Image:  "6704.png",
		Active: true,
		LEDs:   leds,

		Status: &leds[0],
		Link:   getSliceAddr(leds, 1, 5),
	}
}

func GenSUP720() LineCard {

	leds := make([]pixel.Pixel, 9)

	return LineCard{
		Name:   "sup720",
		Image:  "sup720.png",
		Active: true,
		LEDs:   leds,

		Status: &leds[0],
		Link:   getSliceAddr(leds, 6, 9),
		Misc: map[string]*pixel.Pixel{
			"system": &leds[1],
			"active": &leds[2],
			"mgmt":   &leds[3],
			"disk0":  &leds[4],
			"disk1":  &leds[5],
		},
	}
}

func GenBlank() LineCard {

	return LineCard{
		Name:   "blank",
		Image:  "blank.png",
		Active: false,
	}
}
