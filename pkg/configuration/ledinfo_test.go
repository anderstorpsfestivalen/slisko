package configuration

import "testing"

func TestLoadConfigsHaveLedInfo(t *testing.T) {
	cases := []struct {
		path     string
		wantType string
		wantOuts int
	}{
		{"../../configurations/7609.toml", "APA102", 1},
		{"../../configurations/9010.toml", "WS2815", 2},
	}
	for _, tc := range cases {
		conf, err := LoadFromFile(tc.path)
		if err != nil {
			t.Fatalf("%s: load failed: %v", tc.path, err)
		}
		if conf.LedInfo == nil {
			t.Fatalf("%s: LedInfo is nil", tc.path)
		}
		if conf.LedInfo.Type != tc.wantType {
			t.Errorf("%s: type = %q, want %q", tc.path, conf.LedInfo.Type, tc.wantType)
		}
		if len(conf.LedInfo.Mapping) != tc.wantOuts {
			t.Fatalf("%s: got %d outputs, want %d", tc.path, len(conf.LedInfo.Mapping), tc.wantOuts)
		}
		for i, out := range conf.LedInfo.Mapping {
			start, end, err := out.ParseRange()
			if err != nil {
				t.Errorf("%s: output %d range parse: %v", tc.path, i, err)
			}
			if end <= start {
				t.Errorf("%s: output %d range not ascending: %d-%d", tc.path, i, start, end)
			}
		}
	}
}

func TestLedInfoValidation(t *testing.T) {
	bad := []struct {
		name string
		li   LedInfo
		leds int64
	}{
		{"empty type", LedInfo{Type: "", Mapping: []LedOutput{{Gpio: 5, Range: "0-10"}}}, 10},
		{"bad range", LedInfo{Type: "WS2815", Mapping: []LedOutput{{Gpio: 5, Range: "10"}}}, 10},
		{"descending", LedInfo{Type: "WS2815", Mapping: []LedOutput{{Gpio: 5, Range: "10-5"}}}, 10},
		{"out of bounds", LedInfo{Type: "WS2815", Mapping: []LedOutput{{Gpio: 5, Range: "0-100"}}}, 10},
		{"negative gpio", LedInfo{Type: "WS2815", Mapping: []LedOutput{{Gpio: -1, Range: "0-5"}}}, 10},
	}
	for _, tc := range bad {
		if err := tc.li.validate(tc.leds); err == nil {
			t.Errorf("%s: expected validation error, got nil", tc.name)
		}
	}

	ok := LedInfo{Type: "WS2815", Mapping: []LedOutput{{Gpio: 5, Range: "0-100"}, {Gpio: 7, Range: "100-200"}}}
	if err := ok.validate(200); err != nil {
		t.Errorf("valid LedInfo rejected: %v", err)
	}
}
