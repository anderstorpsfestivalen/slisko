package chassi

import "github.com/anderstorpsfestivalen/slisko/pkg/pixel"

func Gen6478() LineCard {

	leds := make([]pixel.Pixel, 49)

	return LineCard{
		Name:  "6478",
		Image: "6478.png",
		LEDs:  leds,

		Status: &leds[0],
		Link:   getSliceAddr(leds, 1, 49),
	}
}

func Gen6704() LineCard {

	leds := make([]pixel.Pixel, 5)

	return LineCard{
		Name:  "6704",
		Image: "6704.png",
		LEDs:  leds,

		Status: &leds[0],
		Link:   getSliceAddr(leds, 1, 5),
	}
}

func GenSUP720() LineCard {

	leds := make([]pixel.Pixel, 9)

	return LineCard{
		Name:  "SUP720",
		Image: "sup720.png",
		LEDs:  leds,

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
