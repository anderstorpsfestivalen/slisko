package configuration

import (
	"encoding/json"
	"fmt"
	"os"

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
	LEDAmount int64
	Linecards []string
	Patterns  []string
	Mapping   []MappingEntry `toml:"mapping"`
	Buttons   []Button       `toml:"buttons"`
}

type Button struct {
	Pin    string   `toml:"pin"`
	Action []string `toml:"action"`
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
