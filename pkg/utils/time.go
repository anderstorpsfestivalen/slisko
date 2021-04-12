package utils

import (
	"math"
	"time"
)

func SinFull(t time.Time, speed float64) float64 {
	return math.Sin(speed * time.Since(t).Seconds())
}

func CosFull(t time.Time, speed float64) float64 {
	return math.Cos(speed * time.Since(t).Seconds())
}

func Sin(t time.Time, speed float64) float64 {
	return (math.Sin(speed*time.Since(t).Seconds()) + 1) / 2
}

func Cos(t time.Time, speed float64) float64 {
	return (math.Cos(speed*time.Since(t).Seconds()) + 1) / 2
}

func Triangle(t time.Time, period float64, amplitude float64) float64 {
	return math.Abs(math.Mod(time.Since(t).Seconds(), period) - amplitude)
}
