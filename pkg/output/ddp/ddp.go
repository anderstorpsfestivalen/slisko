package ddp

import (
	"bytes"
	"encoding/binary"
	"errors"
)

/// Based on http://www.3waylabs.com/ddp/

type DDPHeader struct {
	Flags  byte
	SeqNum byte
	Type   byte
	ID     byte
	Offset uint32
	Length uint16
	Time   uint32 // only present if T flag is set
}

func ParseDDPHeader(data []byte) (*DDPHeader, error) {
	if len(data) < 10 {
		return nil, errors.New("DDP packet header too short")
	}

	header := &DDPHeader{
		Flags:  data[0],
		SeqNum: data[1] & 0x0F,
		Type:   data[2],
		ID:     data[3],
		Offset: binary.BigEndian.Uint32(data[4:8]),
		Length: binary.BigEndian.Uint16(data[8:10]),
	}

	if header.Flags&0x08 != 0 {
		if len(data) < 14 {
			return nil, errors.New("DDP packet header with timecode too short")
		}
		header.Time = binary.BigEndian.Uint32(data[10:14])
	}

	return header, nil
}

func createDDPPacket(flags byte, seqNum byte, dataType byte, id byte, dataOffset uint32, dataLength uint16, timecode uint32, data []byte) []byte {
	// Create a byte buffer to store the packet
	var buffer bytes.Buffer

	// Write the header bytes to the buffer
	header := []byte{flags, ((seqNum << 4) & 0xf0) | (dataType & 0x0f), id}
	binary.Write(&buffer, binary.BigEndian, header)
	binary.Write(&buffer, binary.BigEndian, dataOffset)
	binary.Write(&buffer, binary.BigEndian, dataLength)

	// If the timecode flag is set, write the timecode field to the buffer
	if flags&(1<<3) > 0 {
		binary.Write(&buffer, binary.BigEndian, timecode)
	}

	// If data is provided, write it to the buffer
	if data != nil {
		buffer.Write(data)
	}

	return buffer.Bytes()
}

func (h *DDPHeader) Bytes() []byte {
	var headerLen int
	if h.Flags&0x08 != 0 {
		headerLen = 14
	} else {
		headerLen = 10
	}

	buf := make([]byte, headerLen)

	buf[0] = h.Flags
	buf[1] = (h.SeqNum & 0x0F) | (byte((h.Length >> 8)) & 0xF0)
	buf[2] = h.Type
	buf[3] = h.ID

	binary.BigEndian.PutUint32(buf[4:8], h.Offset)
	binary.BigEndian.PutUint16(buf[8:10], h.Length)

	if h.Flags&0x08 != 0 {
		binary.BigEndian.PutUint32(buf[10:14], h.Time)
	}

	return buf
}
