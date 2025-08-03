package gpio

import (
	"log"
	"os"
	"runtime"
	"strings"

	"github.com/anderstorpsfestivalen/slisko/pkg/configuration"
	"periph.io/x/conn/v3/gpio"
	"periph.io/x/conn/v3/gpio/gpioreg"
	"periph.io/x/host/v3"
)

type GPIOController struct {
}

func NewGPIOController(buttons []configuration.Button) (*GPIOController, error) {

	isLinux := runtime.GOOS == "linux"
	isRaspberryPi := false

	// Check for Raspberry Pi by reading /proc/cpuinfo
	if isLinux {
		if f, err := os.Open("/proc/cpuinfo"); err == nil {
			defer f.Close()
			buf := make([]byte, 4096)
			n, _ := f.Read(buf)
			cpuinfo := string(buf[:n])
			if strings.Contains(cpuinfo, "Raspberry Pi") || strings.Contains(cpuinfo, "BCM") {
				isRaspberryPi = true
			}
		}
	}

	// Check for GPIO by looking for /dev/gpiomem or /dev/gpiochip0
	hasGPIO := false
	if _, err := os.Stat("/dev/gpiomem"); err == nil {
		hasGPIO = true
	} else if _, err := os.Stat("/dev/gpiochip0"); err == nil {
		hasGPIO = true
	}

	// lets load the buttons
	if isLinux && isRaspberryPi && hasGPIO {
		//buttons := gpio.NewGPIOController()
		//buttons.Start()
		if _, err := host.Init(); err != nil {
			if err != nil {
				return nil, err
			}
		}
		for _, button := range buttons {
			p := gpioreg.ByName(button.Pin)
			if p == nil {
				panic("GPIO SETUP: Failed to find " + button.Pin)
			}

			if err := p.In(gpio.PullDown, gpio.BothEdges); err != nil {
				log.Fatal(err)
			}

		}
	}

	return &GPIOController{}, nil
}

func (c *GPIOController) Start() {}
