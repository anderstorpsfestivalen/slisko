package api

import (
	"encoding/json"
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/controller"
	"github.com/gin-contrib/static"
	"github.com/gin-gonic/contrib/cors"
	"github.com/gin-gonic/contrib/ginrus"
	"github.com/gin-gonic/gin"
	"github.com/olahol/melody"
	"github.com/sirupsen/logrus"
	log "github.com/sirupsen/logrus"
)

type API struct {
	router *gin.Engine
	m      *melody.Melody
	port   int

	chassi     *chassi.Chassi
	controller *controller.Controller
}

type UpdateH struct {
	Message string
	Data    []string
}

func New(ch *chassi.Chassi, co *controller.Controller) API {
	return API{
		chassi:     ch,
		controller: co,
		m:          melody.New(),
	}
}

func (a *API) Start(listen string) {
	a.router = gin.Default()

	corsConf := cors.DefaultConfig()
	a.router.Use(cors.New(corsConf))

	logger := logrus.New()
	logger.Level = logrus.WarnLevel
	a.router.Use(ginrus.Ginrus(logger, time.RFC3339, true))
	a.router.Use(static.Serve("/", static.LocalFile("ui/dist", false)))

	//Websocket
	a.router.GET("/ws", func(c *gin.Context) {
		a.m.HandleRequest(c.Writer, c.Request)
	})

	go func() {
		for {
			msg := <-a.controller.Updates
			b, _ := json.Marshal(msg)
			a.m.Broadcast(b)
		}
	}()

	a.m.HandleConnect(func(s *melody.Session) {
		data := a.controller.GetPatternPerCategory()
		b, _ := json.Marshal(data)
		s.Write(b)
	})

	//Chassi
	a.router.GET("/chassi", a.GetChassiConfiguration)

	//Patterns
	a.router.GET("/patterns", a.GetPatterns)
	a.router.GET("/pattern/enable/:pattern", a.EnablePattern)
	a.router.GET("/pattern/disable/:pattern", a.DisablePattern)

	a.router.Run(listen)
}

func (a *API) GetChassiConfiguration(c *gin.Context) {
	c.JSON(200, a.chassi.GetCardOrder())
}

func (a *API) GetPatterns(c *gin.Context) {
	c.JSON(200, a.controller.ListPatterns())
}

func (a *API) EnablePattern(c *gin.Context) {
	pattern := c.Param("pattern")
	if !a.controller.PatternExists(pattern) {
		log.WithField("pattern", pattern).Warn("Pattern does not exist")
		c.JSON(404, gin.H{
			"message": "Could not find pattern",
		})
		return
	}

	a.controller.EnablePattern(pattern)
	c.JSON(202, gin.H{"status": "pattern enabled"})

}

func (a *API) DisablePattern(c *gin.Context) {
	pattern := c.Param("pattern")
	if !a.controller.PatternExists(pattern) {
		log.WithField("pattern", pattern).Warn("Pattern does not exist")
		c.JSON(404, gin.H{
			"message": "Could not find pattern",
		})
		return
	}

	a.controller.DisablePattern(pattern)
	c.JSON(202, gin.H{"status": "pattern disabled"})

}
