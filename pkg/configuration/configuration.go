package configuration

import (
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
