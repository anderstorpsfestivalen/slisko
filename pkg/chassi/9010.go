package chassi

import "github.com/anderstorpsfestivalen/slisko/pkg/pixel"

func GenA9KRSP440SE() LineCard {

	leds := make([]pixel.Pixel, 11)

	l := LineCard{
		Name:   "A9K-RSP440-SE",
		Image:  "a9k-rsp440-se.png",
		Active: true,
		LEDs:   leds,

		Link: getSliceAddr(leds, 0, 1),
		Labeled: map[string]*pixel.Pixel{
			"fail":     &leds[8],
			"crit":     &leds[5],
			"sso":      &leds[2],
			"aco":      &leds[9],
			"maj":      &leds[6],
			"fc_fault": &leds[3],
			"sync":     &leds[10],
			"min":      &leds[7],
			"gps":      &leds[4],
		},
	}

	setManyPixelPositons(l.LEDs, []pixel.Position{

		// SFP
		{X: 57, Y: 187, Size: 5},
		{X: 57, Y: 198, Size: 5},

		// 3x3 block

		// sso
		{X: 57, Y: 857, Size: 4},
		// fc_fault
		{X: 57, Y: 880, Size: 4},
		// gps
		{X: 57, Y: 900, Size: 4},

		// crit
		{X: 43, Y: 857, Size: 4},
		// maj
		{X: 43, Y: 880, Size: 4},
		// min
		{X: 43, Y: 900, Size: 4},

		// fail
		{X: 30, Y: 857, Size: 4},
		// aco
		{X: 30, Y: 880, Size: 4},
		// sync
		{X: 30, Y: 900, Size: 4},
	})

	return l
}

func GenA9KRSP440SE2() LineCard {

	leds := make([]pixel.Pixel, 11)

	l := LineCard{
		Name:   "A9K-RSP440-SE-2",
		Image:  "a9k-rsp440-se.png",
		Active: true,
		LEDs:   leds,

		Link: getSliceAddr(leds, 0, 1),
		Labeled: map[string]*pixel.Pixel{
			"fail":     &leds[9],
			"crit":     &leds[6],
			"sso":      &leds[2],
			"aco":      &leds[7],
			"maj":      &leds[3],
			"fc_fault": &leds[4],
			"sync":     &leds[10],
			"min":      &leds[8],
			"gps":      &leds[5],
		},
	}

	setManyPixelPositons(l.LEDs, []pixel.Position{

		// SFP
		{X: 57, Y: 187, Size: 5},
		{X: 57, Y: 198, Size: 5},

		// 3x3 block

		// sso
		{X: 57, Y: 857, Size: 4},
		// maj
		{X: 43, Y: 880, Size: 4},
		// fc_fault
		{X: 57, Y: 880, Size: 4},

		// gps
		{X: 57, Y: 900, Size: 4},
		// crit
		{X: 43, Y: 857, Size: 4},
		// aco
		{X: 30, Y: 880, Size: 4},

		// min
		{X: 43, Y: 900, Size: 4},
		// fail
		{X: 30, Y: 857, Size: 4},
		// sync
		{X: 30, Y: 900, Size: 4},
	})

	return l
}

func GenA9K8T() LineCard {

	leds := make([]pixel.Pixel, 9)

	l := LineCard{
		Name:   "A9K-8T-L",
		Image:  "a9k-8t-l.png",
		Active: true,
		LEDs:   leds,

		Status: &leds[8],
		Link:   getSliceAddr(leds, 0, 8),
		Labeled: map[string]*pixel.Pixel{
			"status": &leds[8],
		},
	}

	for k, v := range getSliceMap(getSliceAddr(leds, 1, 9), "p") {
		l.Labeled[k] = v
	}

	setManyPixelPositons(l.LEDs, []pixel.Position{

		// First block
		{X: 75, Y: 73, Size: 5},
		{X: 76, Y: 180, Size: 5},
		{X: 76, Y: 280, Size: 5},
		{X: 76, Y: 380, Size: 5},

		// Second block
		{X: 76, Y: 530, Size: 5},
		{X: 76, Y: 635, Size: 5},
		{X: 76, Y: 740, Size: 5},
		{X: 76, Y: 845, Size: 5},

		// Status led
		{X: 73, Y: 985, Size: 5},
	})
	return l
}

func GenA9K40GE() LineCard {

	leds := make([]pixel.Pixel, 41)

	l := LineCard{
		Name:   "A9K-40GE-L",
		Image:  "a9k-40ge-l.png",
		Active: true,
		LEDs:   leds,

		Status: &leds[0],
		Link:   getSliceAddr(leds, 1, 41),
		Labeled: map[string]*pixel.Pixel{
			"status": &leds[0],
		},
	}

	for k, v := range getSliceMap(getSliceAddr(leds, 1, 41), "p") {
		l.Labeled[k] = v
	}

	setManyPixelPositons(l.LEDs, []pixel.Position{
		// Status led
		{X: 75, Y: 985, Size: 5},

		// block of 10 leds
		// between led: 21 px
		// between groups of 2: 18 px
		// Start 78
		{X: 56, Y: 78, Size: 5},
		{X: 56, Y: 99, Size: 5},
		{X: 57, Y: 117, Size: 5},
		{X: 57, Y: 138, Size: 5},
		{X: 57, Y: 156, Size: 5},
		{X: 57, Y: 177, Size: 5},
		{X: 57, Y: 195, Size: 5},
		{X: 57, Y: 216, Size: 5},
		{X: 57, Y: 233, Size: 5},
		{X: 57, Y: 254, Size: 5},

		// Start 290

		{X: 57, Y: 290, Size: 5},
		{X: 57, Y: 311, Size: 5},
		{X: 57, Y: 329, Size: 5},
		{X: 57, Y: 350, Size: 5},
		{X: 57, Y: 368, Size: 5},
		{X: 57, Y: 389, Size: 5},
		{X: 57, Y: 407, Size: 5},
		{X: 57, Y: 428, Size: 5},
		{X: 57, Y: 446, Size: 5},
		{X: 57, Y: 467, Size: 5},

		// Start 531
		{X: 56, Y: 531, Size: 5},
		{X: 56, Y: 552, Size: 5},
		{X: 56, Y: 570, Size: 5},
		{X: 56, Y: 591, Size: 5},
		{X: 56, Y: 609, Size: 5},
		{X: 56, Y: 630, Size: 5},
		{X: 56, Y: 648, Size: 5},
		{X: 56, Y: 669, Size: 5},
		{X: 56, Y: 687, Size: 5},
		{X: 56, Y: 708, Size: 5},

		// Start 744
		{X: 56, Y: 744, Size: 5},
		{X: 56, Y: 765, Size: 5},
		{X: 56, Y: 783, Size: 5},
		{X: 56, Y: 804, Size: 5},
		{X: 56, Y: 822, Size: 5},
		{X: 56, Y: 843, Size: 5},
		{X: 56, Y: 861, Size: 5},
		{X: 56, Y: 882, Size: 5},
		{X: 56, Y: 900, Size: 5},
		{X: 56, Y: 921, Size: 5},
	})
	return l
}
