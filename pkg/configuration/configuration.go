package configuration

import (
	"encoding/json"
	"fmt"
	"os"
	"strconv"
	"strings"

	"github.com/BurntSushi/toml"
)

func LoadFromFile(path string) (ChassiDefiniton, error) {

	dat, err := os.ReadFile(path)
	if err != nil {
		return ChassiDefiniton{}, err
	}

	var conf ChassiDefiniton
	_, err = toml.Decode(string(dat), &conf)

	if err != nil {
		return ChassiDefiniton{}, err
	}

	// Validate configuration before returning
	if err := conf.Validate(); err != nil {
		return ChassiDefiniton{}, fmt.Errorf("invalid configuration in %s: %w", path, err)
	}

	return conf, nil
}

type ChassiDefiniton struct {
	LEDAmount      int64
	Linecards      []string
	Patterns       []string
	Mapping        []MappingEntry `toml:"mapping"`
	Buttons        []Button       `toml:"buttons"`
	TrafficShaper  *TrafficShaper `toml:"traffic_shaper,omitempty"`
	Output         *Output        `toml:"output,omitempty"`
	LedInfo        *LedInfo       `toml:"ledinfo,omitempty"`
}

// LedInfo describes the physical LED hardware: the pixel chip type and the
// mapping of GPIO data pins to contiguous pixel index ranges of the flattened
// strand. It is consumed both by direct hardware output paths and by the
// firmware "baker" (cmd/baker) that bakes a config onto an ESP32 board.
type LedInfo struct {
	// Type is the pixel chip, e.g. "WS2815", "WS2812", "WS2811", "APA102".
	Type string `toml:"type"`
	// Mapping assigns each data GPIO a contiguous range of the strand.
	Mapping []LedOutput `toml:"mapping"`
}

// LedOutput maps a single data GPIO to a half-open pixel index range.
type LedOutput struct {
	Gpio  int    `toml:"gpio"`
	Range string `toml:"range"` // "start-end", half-open [start, end)
}

// ParseRange parses the "start-end" range into half-open bounds [start, end).
func (o LedOutput) ParseRange() (start int, end int, err error) {
	parts := strings.SplitN(o.Range, "-", 2)
	if len(parts) != 2 {
		return 0, 0, fmt.Errorf("range %q must be in the form \"start-end\"", o.Range)
	}
	start, err = strconv.Atoi(strings.TrimSpace(parts[0]))
	if err != nil {
		return 0, 0, fmt.Errorf("range %q: invalid start: %w", o.Range, err)
	}
	end, err = strconv.Atoi(strings.TrimSpace(parts[1]))
	if err != nil {
		return 0, 0, fmt.Errorf("range %q: invalid end: %w", o.Range, err)
	}
	return start, end, nil
}

type Button struct {
	Pin    string   `toml:"pin"`
	Action []string `toml:"action"`
}

type Output struct {
	DDP *DDPOutput `toml:"ddp,omitempty"`
}

type DDPOutput struct {
	Host string `toml:"host"`
	Port int    `toml:"port"`
}

// GetDDPAddress returns the full DDP address (host:port) or empty string if not configured
func (o *Output) GetDDPAddress() string {
	if o == nil || o.DDP == nil || o.DDP.Host == "" {
		return ""
	}
	return fmt.Sprintf("%s:%d", o.DDP.Host, o.DDP.Port)
}

// HasDDP returns true if DDP output is configured
func (o *Output) HasDDP() bool {
	return o != nil && o.DDP != nil && o.DDP.Host != ""
}

type TrafficShaper struct {
	Enabled    bool    `toml:"enabled"`
	PeakStart  int     `toml:"peak_start"`   // Hour of day (0-23), e.g., 17 for 5 PM
	PeakEnd    int     `toml:"peak_end"`     // Hour of day (0-23), e.g., 22 for 10 PM
	LowStart   int     `toml:"low_start"`    // Hour of day (0-23), e.g., 2 for 2 AM
	LowEnd     int     `toml:"low_end"`      // Hour of day (0-23), e.g., 7 for 7 AM
	PeakFactor float64 `toml:"peak_factor"`  // Multiplier during peak hours (1.0 = baseline/maximum as defined in styles)
	LowFactor  float64 `toml:"low_factor"`   // Multiplier during low hours (0.0-1.0, e.g., 0.2 = 20% of peak)
}

// DefaultTrafficShaper returns a TrafficShaper with sensible defaults
func DefaultTrafficShaper() *TrafficShaper {
	return &TrafficShaper{
		Enabled:    true,
		PeakStart:  17, // 5 PM
		PeakEnd:    22, // 10 PM
		LowStart:   2,  // 2 AM
		LowEnd:     7,  // 7 AM
		PeakFactor: 1.0, // Peak is baseline (maximum blinking as defined in styles)
		LowFactor:  0.2, // Low period is 20% of peak
	}
}

// GetOrDefault returns the traffic shaper or a default one if not configured
func (c *ChassiDefiniton) GetTrafficShaper() *TrafficShaper {
	if c.TrafficShaper != nil {
		return c.TrafficShaper
	}
	return DefaultTrafficShaper()
}

type MappingEntry struct {
	Gen  *int `toml:"gen"`
	Card *int `toml:"card"`
}

func (m *MappingEntry) IsGen() bool {
	return m.Gen != nil
}

func (m *MappingEntry) IsCard() bool {
	return m.Card != nil
}

// Print outputs the configuration in a formatted JSON format
func (c *ChassiDefiniton) Print() error {
	configJSON, err := json.MarshalIndent(c, "", "  ")
	if err != nil {
		fmt.Printf("Error formatting configuration: %v\n", err)
		fmt.Printf("Raw configuration: %+v\n", c)
		return err
	}
	fmt.Printf("%s\n", configJSON)
	return nil
}

// PrintWithSource outputs the configuration with source file information
func (c *ChassiDefiniton) PrintWithSource(configFile string) error {
	fmt.Printf("Configuration loaded from: %s\n\n", configFile)
	return c.Print()
}

func (c *ChassiDefiniton) UsesButtons() bool {
	return len(c.Buttons) > 0
}

// Validate checks that the configuration is internally consistent and safe to use
func (c *ChassiDefiniton) Validate() error {
	numLinecards := len(c.Linecards)

	// Validate mapping entries
	for i, mapping := range c.Mapping {
		if err := validateMappingEntry(mapping, i, numLinecards); err != nil {
			return err
		}
	}

	// Validate LED hardware info
	if c.LedInfo != nil {
		if err := c.LedInfo.validate(c.LEDAmount); err != nil {
			return err
		}
	}

	return nil
}

// validate checks the LED type is set and every output range is well-formed,
// non-empty, ascending, and within [0, ledAmount].
func (l *LedInfo) validate(ledAmount int64) error {
	if strings.TrimSpace(l.Type) == "" {
		return fmt.Errorf("ledinfo: type must be set (e.g. \"WS2815\", \"APA102\")")
	}

	for i, out := range l.Mapping {
		start, end, err := out.ParseRange()
		if err != nil {
			return fmt.Errorf("ledinfo.mapping[%d]: %w", i, err)
		}
		if out.Gpio < 0 {
			return fmt.Errorf("ledinfo.mapping[%d]: gpio %d is negative", i, out.Gpio)
		}
		if start < 0 {
			return fmt.Errorf("ledinfo.mapping[%d]: range start %d is negative", i, start)
		}
		if end <= start {
			return fmt.Errorf("ledinfo.mapping[%d]: range %q is empty or descending (end must be > start)", i, out.Range)
		}
		if int64(end) > ledAmount {
			return fmt.Errorf("ledinfo.mapping[%d]: range end %d exceeds LEDAmount %d", i, end, ledAmount)
		}
	}

	return nil
}

// validateMappingEntry checks a single mapping entry for validity
func validateMappingEntry(m MappingEntry, index int, numLinecards int) error {
	// Check if mapping entry has at least one field set
	if !m.IsCard() && !m.IsGen() {
		return fmt.Errorf("mapping[%d]: must specify either 'card' or 'gen'", index)
	}

	// Validate card index if present
	if m.IsCard() {
		if err := validateCardIndex(*m.Card, index, numLinecards); err != nil {
			return err
		}
	}

	// Validate gen index if present (must be non-negative)
	if m.IsGen() {
		if *m.Gen < 0 {
			return fmt.Errorf("mapping[%d]: gen index %d is negative, must be >= 0", index, *m.Gen)
		}
	}

	return nil
}

// validateCardIndex checks that a card index is within valid bounds
func validateCardIndex(cardIndex int, mappingIndex int, numLinecards int) error {
	if cardIndex < 0 {
		return fmt.Errorf("mapping[%d]: card index %d is negative, must be >= 0", mappingIndex, cardIndex)
	}

	if cardIndex >= numLinecards {
		return fmt.Errorf("mapping[%d]: card index %d is out of bounds (config has %d linecards, valid indices: 0-%d)",
			mappingIndex, cardIndex, numLinecards, numLinecards-1)
	}

	return nil
}
