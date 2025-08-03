package gpio

import (
	"log"
	"os"
	"runtime"
	"strings"

	"periph.io/x/host/v3"
)

type GPIOController struct {
}

func NewGPIOController() (*GPIOController, error) {
	if _, err := host.Init(); err != nil {
		log.Fatal(err)
	}

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

	if isLinux && isRaspberryPi && hasGPIO {
		//buttons := gpio.NewGPIOController()
		//buttons.Start()
	}

	return &GPIOController{}, nil
}

func (c *GPIOController) Start() {}
