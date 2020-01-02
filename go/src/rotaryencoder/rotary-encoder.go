package rotaryencoder

import (
	"github.com/wfd3/go-rpio"
  "sync"
)

var (
  prevNextCode int8 = 0
  store int16 = 0
  lock sync.Mutex
)

//https://www.best-microcontroller-projects.com/rotary-encoder.html
func Read(dataPin rpio.Pin, clickPin rpio.Pin) int {
  var rotTable [16]int8 = [16]int8{0,1,1,0,1,0,0,1,1,0,0,1,0,1,1,0}

  lock.Lock()
  prevNextCode <<= 2
  if (dataPin.Read() == 1) {
    prevNextCode |= 0x02
  }

  if (clickPin.Read() == 1) {
    prevNextCode |= 0x01
  }
  prevNextCode &= 0x0f

  if (rotTable[prevNextCode] == 1) {
    store <<= 4
    store |= int16(prevNextCode)

    if ((store&0xff)==0x2b) {
    lock.Unlock()
      return -1
    }
    if ((store&0xff)==0x17) {
      lock.Unlock()
      return 1
    }
  }
  lock.Unlock()
  return 0
}
