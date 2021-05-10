package controller

type Broker struct {
	stopCh    chan struct{}
	publishCh chan bool
	subCh     chan chan bool
	unsubCh   chan chan bool
}

func NewBroker() *Broker {
	return &Broker{
		stopCh:    make(chan struct{}),
		publishCh: make(chan bool, 1),
		subCh:     make(chan chan bool, 1),
		unsubCh:   make(chan chan bool, 1),
	}
}

func (b *Broker) Start() {
	subs := map[chan bool]struct{}{}
	for {
		select {
		case <-b.stopCh:
			return
		case msgCh := <-b.subCh:
			subs[msgCh] = struct{}{}
		case msgCh := <-b.unsubCh:
			delete(subs, msgCh)
		case msg := <-b.publishCh:
			for msgCh := range subs {
				// msgCh is buffered, use non-blocking send to protect the broker:
				select {
				case msgCh <- msg:
				default:
				}
			}
		}
	}
}

func (b *Broker) Stop() {
	close(b.stopCh)
}

func (b *Broker) Subscribe() chan bool {
	msgCh := make(chan bool, 5)
	b.subCh <- msgCh
	return msgCh
}

func (b *Broker) Unsubscribe(msgCh chan bool) {
	b.unsubCh <- msgCh
}

func (b *Broker) Publish(msg bool) {
	b.publishCh <- msg
}
