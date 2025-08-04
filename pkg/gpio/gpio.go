package gpio

import (
	"context"
	"fmt"
	"log"
	"os"
	"runtime"
	"strings"
	"sync"
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/configuration"
	"periph.io/x/conn/v3/gpio"
	"periph.io/x/conn/v3/gpio/gpioreg"
	"periph.io/x/host/v3"
)

type GPIOController struct {
	ab     []ActiveButton
	ctx    context.Context
	cancel context.CancelFunc
	wg     sync.WaitGroup
}

type ActiveButton struct {
	p     gpio.PinIO
	Scene string
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
			return nil, err
		}
		var ab []ActiveButton

		for _, button := range buttons {
			p := gpioreg.ByName(button.Pin)
			if p == nil {
				panic("GPIO SETUP: Failed to find " + button.Pin)
			}

			if err := p.In(gpio.PullDown, gpio.BothEdges); err != nil {
				log.Fatal(err)
			}

			ab = append(ab, ActiveButton{
				p:     p,
				Scene: button.Action,
			})
		}

		ctx, cancel := context.WithCancel(context.Background())
		ctrl := &GPIOController{
			ab:     ab,
			ctx:    ctx,
			cancel: cancel,
		}

		go ctrl.start()

		return ctrl, nil

	}

	return &GPIOController{}, fmt.Errorf("could not initalize GPIO")
}

func (c *GPIOController) start() {
	// Start a goroutine for each button to monitor hardware interrupts
	for i := range c.ab {
		c.wg.Add(1)
		go c.monitorButton(&c.ab[i])
	}

	// Wait for all button monitors to finish
	c.wg.Wait()
}

// monitorButton waits for hardware interrupts on a specific button
func (c *GPIOController) monitorButton(button *ActiveButton) {
	defer c.wg.Done()

	log.Printf("Starting interrupt monitoring for button on pin %s (action: %s)", button.p, button.Scene)

	lastTrigger := time.Now()
	debounceTime := 50 * time.Millisecond // Prevent bounce

	for {
		select {
		case <-c.ctx.Done():
			// Graceful shutdown
			log.Printf("Stopping monitoring for button on pin %s", button.p)
			return
		default:
			// Wait for hardware interrupt (falling edge = button press)
			// This is CPU efficient as it relies on hardware interrupts
			if button.p.WaitForEdge(-1) { // -1 means wait indefinitely
				now := time.Now()

				// Debounce: ignore rapid successive triggers
				if now.Sub(lastTrigger) < debounceTime {
					continue
				}

				// Check if this is a falling edge (button press)
				if button.p.Read() == gpio.Low {
					lastTrigger = now
					log.Printf("Button pressed on pin %s, triggering action: %s", button.p, button.Scene)

					// Fire the button action with access to the controller
					c.onButtonPress(button)
				}
			}
		}
	}
}

// onButtonPress handles the button press event
func (c *GPIOController) onButtonPress(button *ActiveButton) {
	// This function fires when a button is pressed
	// You have access to the GPIOController instance (c) and the specific button

	log.Printf("Executing action '%s' for button on pin %s", button.Scene, button.p)

	// TODO: Add your button action logic here
	// For example, you might want to trigger a scene change, pattern switch, etc.
	// You can access other parts of your application through the controller instance

	// Example of what you might do:
	// - Switch LED patterns
	// - Change scenes
	// - Send API calls
	// - Update application state
}

// Stop gracefully shuts down the GPIO controller
func (c *GPIOController) Stop() {
	log.Println("Stopping GPIO controller...")
	if c.cancel != nil {
		c.cancel()
	}
	c.wg.Wait()
	log.Println("GPIO controller stopped")
}
