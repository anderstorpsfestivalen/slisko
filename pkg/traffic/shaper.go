package traffic

import (
	"math"
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/configuration"
)

// Shaper calculates traffic intensity based on time of day
type Shaper struct {
	config *configuration.TrafficShaper
}

// NewShaper creates a new traffic shaper from configuration
func NewShaper(config *configuration.TrafficShaper) *Shaper {
	return &Shaper{
		config: config,
	}
}

// GetIntensity returns the current traffic intensity multiplier (0.0 to peak factor)
// Uses a sinewave pattern with peak during evening hours and low during night/early morning
func (s *Shaper) GetIntensity() float64 {
	if !s.config.Enabled {
		return 1.0
	}

	now := time.Now()
	hour := now.Hour()
	minute := now.Minute()

	// Convert to fractional hour for smoother transitions
	fractionalHour := float64(hour) + float64(minute)/60.0

	// Calculate intensity based on time of day
	return s.calculateIntensity(fractionalHour)
}

// calculateIntensity computes traffic intensity using a sinewave pattern
func (s *Shaper) calculateIntensity(hourOfDay float64) float64 {
	peakStart := float64(s.config.PeakStart)
	peakEnd := float64(s.config.PeakEnd)
	lowStart := float64(s.config.LowStart)
	lowEnd := float64(s.config.LowEnd)

	peakMid := (peakStart + peakEnd) / 2.0
	lowMid := (lowStart + lowEnd) / 2.0

	// Handle wrap-around midnight for low period
	if lowEnd < lowStart {
		lowMid = (lowStart + lowEnd + 24) / 2.0
		if lowMid >= 24 {
			lowMid -= 24
		}
	}

	// Create a sinewave that peaks at peak_mid and hits minimum at low_mid
	// The sinewave completes a full cycle in 24 hours

	// Shift so that peak is at peak_mid
	shiftedHour := hourOfDay - peakMid
	if shiftedHour < 0 {
		shiftedHour += 24
	}
	if shiftedHour >= 24 {
		shiftedHour -= 24
	}

	// Convert to radians (0 to 2π over 24 hours)
	// We want the peak at hour 0 (which is peak_mid after shift)
	angle := (shiftedHour / 24.0) * 2.0 * math.Pi

	// Sine wave: -1 to 1, with peak at 0 radians
	// We use cosine so peak is at 0
	sineValue := math.Cos(angle)

	// Map sine value (-1 to 1) to (low_factor to peak_factor)
	minFactor := s.config.LowFactor
	maxFactor := s.config.PeakFactor

	intensity := ((sineValue + 1.0) / 2.0) * (maxFactor - minFactor) + minFactor

	return intensity
}

// GetScaledDuration scales a duration by current traffic intensity
// Higher intensity = shorter durations (more frequent activity)
func (s *Shaper) GetScaledDuration(base time.Duration) time.Duration {
	intensity := s.GetIntensity()
	if intensity <= 0 {
		intensity = 0.1 // Prevent division by zero
	}
	// Higher intensity means shorter intervals (inverse relationship)
	return time.Duration(float64(base) / intensity)
}

// GetScaledInterval returns min and max durations scaled by traffic intensity
func (s *Shaper) GetScaledInterval(minBase, maxBase time.Duration) (time.Duration, time.Duration) {
	intensity := s.GetIntensity()
	if intensity <= 0 {
		intensity = 0.1
	}
	return time.Duration(float64(minBase) / intensity), time.Duration(float64(maxBase) / intensity)
}
