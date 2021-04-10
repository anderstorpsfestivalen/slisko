package api

import (
	"fmt"
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/controller"
	"github.com/gin-gonic/contrib/cors"
	"github.com/gin-gonic/contrib/ginrus"
	"github.com/gin-gonic/gin"
	"github.com/sirupsen/logrus"
	log "github.com/sirupsen/logrus"
)

type API struct {
	router *gin.Engine
	port   int

	chassi     *chassi.Chassi
	controller *controller.Controller
}

func New(ch *chassi.Chassi, co *controller.Controller) API {
	return API{
		chassi:     ch,
		controller: co,
	}
}

func (a *API) Start(listen string) {
	a.router = gin.Default()

	corsConf := cors.DefaultConfig()
	a.router.Use(cors.New(corsConf))

	logger := logrus.New()
	logger.Level = logrus.WarnLevel
	a.router.Use(ginrus.Ginrus(logger, time.RFC3339, true))

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
	fmt.Println(pattern)
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
