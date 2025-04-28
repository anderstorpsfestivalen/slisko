package controller

import (
	"time"

	"github.com/anderstorpsfestivalen/slisko/patterns"
	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	log "github.com/sirupsen/logrus"
)

var pt = map[string]patterns.Pattern{
	"blink48ports": &patterns.Blink48Ports{},
	"greenstatus":  &patterns.GreenStatus{},
	"redstatus":    &patterns.RedStatus{},
	"strobe":       &patterns.Strobe{},
	"sup720":       &patterns.SUP720{},
	"x6704":        &patterns.X6704{},
	"colorcycler":  &patterns.Colorcycler{},
	"snake":        &patterns.Snake{},
	"mapper":       &patterns.Mapper{},
	"a9k-8t-l":     &patterns.A9K8TL{},
	"a9k-40ge-l":   &patterns.A9K40GE{},
}

type Controller struct {
	c *chassi.Chassi

	start  time.Time
	ticker *time.Ticker

	framerate      int
	frame          int64
	Updates        chan map[string][]TP
	activePatterns []patterns.Pattern

	FrameBroker *Broker
}

func New(c *chassi.Chassi) Controller {

	//Create patterns
	for _, p := range pt {
		p.Bootstrap(c)
	}

	return Controller{
		c: c,

		start:       time.Now(),
		Updates:     make(chan map[string][]TP, 20),
		FrameBroker: NewBroker(),
	}
}

func (ctrl *Controller) Start(framerate int) {
	ctrl.framerate = framerate
	go ctrl.render()
	go ctrl.FrameBroker.Start()
}

func (ctrl *Controller) Stop() {
	ctrl.ticker.Stop()
	ctrl.frame = 0
}

func (ctrl *Controller) ListPatterns() []patterns.PatternInfo {
	d := []patterns.PatternInfo{}
	for _, p := range pt {
		d = append(d, p.Info())
	}
	return d
}

type TP struct {
	PatternName string
	Category    string
	Enabled     bool
}

func (ctrl *Controller) GetPatternPerCategory() map[string][]TP {
	m := make(map[string][]TP)
	for _, p := range pt {
		enabled := ctrl.CheckIfActive(p.Info().Name)
		m[p.Info().Category] = append(m[p.Info().Category], TP{
			PatternName: p.Info().Name,
			Category:    p.Info().Category,
			Enabled:     enabled,
		})
	}
	return m
}

func (ctrl *Controller) PatternExists(p string) bool {
	if _, ok := pt[p]; ok {
		return true
	}
	return false
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
		//Disable patterns in same category
		for i, a := range ctrl.activePatterns {
			if a.Info().Category == pattern.Info().Category {
				if a.Info().Category != "misc" {
					ctrl.activePatterns = remove(ctrl.activePatterns, i)
				}
			}
		}

		ctrl.activePatterns = append(ctrl.activePatterns, pattern)
		log.Info("Activated pattern: " + pattern.Info().Name)
	}

	ctrl.Updates <- ctrl.GetPatternPerCategory()
}

func (ctrl *Controller) DisablePattern(p string) {
	for i, a := range ctrl.activePatterns {
		if a.Info().Name == p {
			ctrl.activePatterns = remove(ctrl.activePatterns, i)
			for _, l := range ctrl.c.LEDs {
				l.SetColor(0.0, 0.0, 0.0)
			}
			log.Info("Disabled pattern: " + a.Info().Name)
			return
		}
	}
	ctrl.Updates <- ctrl.GetPatternPerCategory()
}

func (ctrl *Controller) CheckIfActive(p string) bool {
	for _, a := range ctrl.activePatterns {
		if a.Info().Name == p {
			return true
		}
	}
	return false
}

func (ctrl *Controller) render() {
	ctrl.ticker = time.NewTicker((1000 / time.Duration(ctrl.framerate)) * time.Millisecond)
	for {
		_ = <-ctrl.ticker.C
		info := patterns.RenderInfo{
			Start: ctrl.start,
			Frame: ctrl.frame,
		}
		for _, p := range ctrl.activePatterns {
			if p.Info().Category == "global" {
				p.Render(info, ctrl.c)
				continue
			}
		}
		for _, p := range ctrl.activePatterns {
			p.Render(info, ctrl.c)
		}
		ctrl.FrameBroker.Publish(true)

		ctrl.frame = ctrl.frame + 1
	}
}

func remove(slice []patterns.Pattern, s int) []patterns.Pattern {
	return append(slice[:s], slice[s+1:]...)
}
