package controller

import (
	"time"

	"github.com/anderstorpsfestivalen/slisko/patterns"
	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	log "github.com/sirupsen/logrus"
)

var pt = map[string]patterns.Pattern{
	"blinkports": &patterns.BlinkPorts{},
}

type Controller struct {
	c         *chassi.Chassi
	framerate int

	activePatterns []patterns.Pattern
}

func New(c *chassi.Chassi) Controller {
	return Controller{
		c: c,
	}
}

func (ctrl *Controller) Start(framerate int) {
	ctrl.framerate = framerate
	go ctrl.render()
}

func (ctrl *Controller) ListPatterns() []patterns.PatternInfo {
	d := []patterns.PatternInfo{}
	for _, p := range pt {
		d = append(d, p.Info())
	}
	return d
}

func (ctrl *Controller) EnablePattern(p string) {

	//Check if pattern is already enabled
	for _, a := range ctrl.activePatterns {
		if a.Info().Name == p {
			return
		}
	}

	//Add pattern to render list
	if pattern, ok := pt[p]; ok {
		ctrl.activePatterns = append(ctrl.activePatterns, pattern)
		log.Info("Activated pattern: " + pattern.Info().Name)
	}
}

func (ctrl *Controller) DisablePattern(p string) {
	for i, a := range ctrl.activePatterns {
		if a.Info().Name == p {
			ctrl.activePatterns = remove(ctrl.activePatterns, i)
			log.Info("Disabled pattern: " + a.Info().Name)
			return
		}
	}
}

func (ctrl *Controller) render() {
	ticker := time.NewTicker((1000 / time.Duration(ctrl.framerate)) * time.Millisecond)
	for {
		_ = <-ticker.C
		for _, p := range ctrl.activePatterns {
			p.Render(ctrl.c)
		}
	}
}

func remove(slice []patterns.Pattern, s int) []patterns.Pattern {
	return append(slice[:s], slice[s+1:]...)
}
