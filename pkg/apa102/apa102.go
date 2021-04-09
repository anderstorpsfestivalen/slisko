package apa102

import (
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
)

type APA102 struct {
	c         *chassi.Chassi
	framerate int
}

func New(c *chassi.Chassi, framerate int) *APA102 {
	return &APA102{
		c:         c,
		framerate: framerate,
	}
}

func (a *APA102) Start() {
	ticker := time.NewTicker((1000 / time.Duration(a.framerate)) * time.Millisecond)
	go func() {
		for {
			_ = <-ticker.C
			//fmt.Println("paint")
			//fmt.Println(a.c.LEDs[0])
		}
	}()
}
