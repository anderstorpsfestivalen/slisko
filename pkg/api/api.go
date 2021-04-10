package api

import (
	"time"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
	"github.com/anderstorpsfestivalen/slisko/pkg/controller"
	"github.com/gin-gonic/contrib/cors"
	"github.com/gin-gonic/contrib/ginrus"
	"github.com/gin-gonic/gin"
	"github.com/sirupsen/logrus"
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

	a.router.GET("/chassi", a.GetChassiConfiguration)

	a.router.Run(listen)
}

func (a *API) GetChassiConfiguration(c *gin.Context) {
	c.JSON(200, a.chassi.GetCardOrder())
}
