package utils

import "math/rand"

func Crush(v float64, threshold float64) float64 {
	if v > threshold {
		return 1
	} else {
		return v
	}
}

func Threshold(v float64, threshold float64) float64 {
	if v > threshold {
		return v
	} else {
		return 0
	}
}

func Trigger(v float64, trigger bool) float64 {
	if trigger {
		return v
	} else {
		return 0.0
	}
}

func Random(min float64, max float64) float64 {
	return min + rand.Float64()*(max-min)
}

func Square(v float64) float64 {
	if v > 0.0 {
		return 1.0
	} else {
		return 0.0
	}
}
