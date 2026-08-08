package socketapa

import (
	"fmt"
	"log"
	"net/url"

	"github.com/anderstorpsfestivalen/slisko/pkg/pixel"
	"github.com/gorilla/websocket"
)

type SocketAPA struct {
	addr string
	conn *websocket.Conn

	mapping       []*pixel.Pixel
	renderTrigger chan bool

	outputBuf []byte

	initated bool
}

func New(addr string, numPixels int64, trigger chan bool) (*SocketAPA, error) {

	u := url.URL{Scheme: "ws", Host: addr, Path: "/"}
	c, _, err := websocket.DefaultDialer.Dial(u.String(), nil)
	if err != nil {
		log.Fatal("dial:", err)
	}
	//defer c.Close()

	return &SocketAPA{
		addr:          addr,
		conn:          c,
		renderTrigger: trigger,
		outputBuf:     make([]byte, numPixels*3),
		initated:      true,
	}, nil
}

func (a *SocketAPA) Run() {
	for {
		<-a.renderTrigger
		for i, l := range a.mapping {
			a.outputBuf[i*3] = pixel.Clamp255(l.R * 255)
			a.outputBuf[i*3+1] = pixel.Clamp255(l.G * 255)
			a.outputBuf[i*3+2] = pixel.Clamp255(l.B * 255)
		}

		if a.initated {
			tmp := append([]byte(nil), a.outputBuf...)
			a.send(tmp)
		}
	}
}

func (a *SocketAPA) send(b []byte) {
	err := a.conn.WriteMessage(websocket.BinaryMessage, b)
	if err != nil {
		fmt.Println("fuck: ", err)
	}
}

func (a *SocketAPA) Map(nm []pixel.Pixel) {
	for z, _ := range nm {
		a.mapping = append(a.mapping, &nm[z])
	}
}

func (a *SocketAPA) GetMap() *[]*pixel.Pixel {
	return &a.mapping
}

func GenEmpty(num int) []pixel.Pixel {
	lp := []pixel.Pixel{}

	for i := 0; i < num; i++ {
		lp = append(lp, pixel.Pixel{})
	}
	return lp
}

func (a *SocketAPA) Clear() {
	for i, _ := range a.outputBuf {
		a.outputBuf[i] = 0
	}
}

func (a *SocketAPA) Close() {
}
