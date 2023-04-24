package ddp

import (
	"fmt"
	"log"
	"testing"
)

func TestPacket(t *testing.T) {
	data := []byte{0x41, 0x12, 0x01, 0x01, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x0C}
	header, err := ParseDDPHeader(data)
	if err != nil {
		log.Fatal(err)
	}
	dv := fmt.Sprintf("Flags: 0x%02X, SeqNum: %d, Type: %d, ID: %d, Offset: %d, Length: %d",
		header.Flags, header.SeqNum, header.Type, header.ID, header.Offset, header.Length)

	t.Log(dv)
}
