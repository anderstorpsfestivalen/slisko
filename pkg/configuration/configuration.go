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
	Pin    int    `toml:"pin"`
	Action string `toml:"action"`
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
