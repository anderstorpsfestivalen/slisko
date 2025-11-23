package patterns

import (
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/faker"
	"github.com/anderstorpsfestivalen/slisko/pkg/pixel"
	"github.com/anderstorpsfestivalen/slisko/pkg/traffic"
)

// GlobalTrafficShaper is the global traffic shaper instance used by all patterns
var GlobalTrafficShaper *traffic.Shaper

// SetGlobalTrafficShaper sets the global traffic shaper for all patterns
func SetGlobalTrafficShaper(shaper *traffic.Shaper) {
	GlobalTrafficShaper = shaper
}

// GetTrafficShaper returns the global traffic shaper, or nil if not set
func GetTrafficShaper() *traffic.Shaper {
	return GlobalTrafficShaper
}

type PatternInfo struct {
	Name     string
	Category string
}

type RenderInfo struct {
	Start time.Time
	Frame int64
}

type Pattern interface {
	Render(info RenderInfo, c *chassi.Chassi)
	Info() PatternInfo
	Bootstrap(c *chassi.Chassi)
}

type mapPort struct {
	faker faker.Fake
	port  *pixel.Pixel
}
