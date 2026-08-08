package patterns

import (
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/faker"
	"github.com/anderstorpsfestivalen/slisko/pkg/pixel"
	"github.com/anderstorpsfestivalen/slisko/pkg/traffic"
	"github.com/anderstorpsfestivalen/slisko/pkg/utils"
)

// BlinkStyle defines how a linecard's ports should blink
type BlinkStyle struct {
	// Timing parameters for the blink pattern (base values, scaled by traffic)
	MinInterval time.Duration
	MaxInterval time.Duration
	MinBlink    time.Duration
	MaxBlink    time.Duration

	// Blinker parameters
	MinBlinks float64
	MaxBlinks float64
	MinCycle  time.Duration
	MaxCycle  time.Duration

	// Color parameters for different speeds
	SlowColor ColorStyle
	FastColor ColorStyle
	DeadColor ColorStyle

	// Dead port probability (0.0-1.0, e.g., 0.067 for ~1 in 15)
	DeadPortChance float64

	// Slow speed probability (0.0-1.0, e.g., 0.2 for 20% slow)
	SlowSpeedChance float64
}

// ColorStyle defines RGB color values (0.0-1.0)
type ColorStyle struct {
	RMult float64
	GMult float64
	BMult float64
}

// PortState represents the state of a single port
type PortState struct {
	faker faker.Fake
	port  *pixel.Pixel
	style ColorStyle
	isDead bool
}

// CreatePort creates a new port with this blink style
func (bs *BlinkStyle) CreatePort(port *pixel.Pixel, shaper *traffic.Shaper) PortState {
	// Check if this should be a dead port
	if randFloat() < bs.DeadPortChance {
		return PortState{
			port:   port,
			style:  bs.DeadColor,
			isDead: true,
		}
	}

	// Determine speed (slow or fast)
	style := bs.FastColor
	if randFloat() < bs.SlowSpeedChance {
		style = bs.SlowColor
	}

	// Scale timing based on traffic
	minInterval, maxInterval := shaper.GetScaledInterval(bs.MinInterval, bs.MaxInterval)
	minBlink, maxBlink := shaper.GetScaledInterval(bs.MinBlink, bs.MaxBlink)

	return PortState{
		faker: faker.NewRandomInterval(
			minInterval,
			maxInterval,
			minBlink,
			maxBlink,
			faker.NewRandomBlinker(bs.MinBlinks, bs.MaxBlinks, bs.MinCycle, bs.MaxCycle),
		),
		port:   port,
		style:  style,
		isDead: false,
	}
}

// Render renders a single port's state
func (ps *PortState) Render() {
	if ps.isDead {
		ps.port.SetClamped(ps.style.RMult, ps.style.GMult, ps.style.BMult)
		return
	}

	v := utils.Invert(ps.faker.Trig())
	ps.port.SetClamped(v*ps.style.RMult, v*ps.style.GMult, v*ps.style.BMult)
}

// Helper function for random floats
func randFloat() float64 {
	return utils.Random(0.0, 1.0)
}

// Predefined blink styles for different linecard types

// ASR9000Style returns the blink style for ASR9000 40GE linecards
func ASR9000Style() BlinkStyle {
	return BlinkStyle{
		MinInterval:     50 * time.Millisecond,
		MaxInterval:     7 * time.Second,
		MinBlink:        50 * time.Millisecond,
		MaxBlink:        12 * time.Second,
		MinBlinks:       15,
		MaxBlinks:       40,
		MinCycle:        1 * time.Second,
		MaxCycle:        10 * time.Second,
		SlowColor:       ColorStyle{RMult: 1.0, GMult: 0.6, BMult: 0.0},
		FastColor:       ColorStyle{RMult: 0.3, GMult: 1.0, BMult: 0.0},
		DeadColor:       ColorStyle{RMult: 1.0, GMult: 0.0, BMult: 0.0},
		DeadPortChance:  0.067, // ~1 in 15
		SlowSpeedChance: 0.2,   // 20% slow
	}
}

// Cisco7609Style returns the blink style for Cisco 7609 linecards
func Cisco7609Style() BlinkStyle {
	return BlinkStyle{
		MinInterval:     100 * time.Millisecond,
		MaxInterval:     7 * time.Second,
		MinBlink:        100 * time.Millisecond,
		MaxBlink:        12 * time.Second,
		MinBlinks:       15,
		MaxBlinks:       40,
		MinCycle:        1 * time.Second,
		MaxCycle:        10 * time.Second,
		SlowColor:       ColorStyle{RMult: 1.0, GMult: 0.5, BMult: 0.0},
		FastColor:       ColorStyle{RMult: 0.3, GMult: 1.0, BMult: 0.0},
		DeadColor:       ColorStyle{RMult: 1.0, GMult: 0.0, BMult: 0.0},
		DeadPortChance:  0.0,   // No dead ports
		SlowSpeedChance: 0.2,   // 20% slow
	}
}
