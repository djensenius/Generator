package main

import (
	"fmt"
	"github.com/wfd3/go-rpio"
  "github.com/hypebeast/go-osc/osc"
	"os"
  "rotaryencoder"
)

var (
  counter = 0
)

func main() {
  if err := rpio.Open(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	defer rpio.Close()
  pin := rpio.Pin(21)
  freqPinA := rpio.Pin(20);
  freqPinB := rpio.Pin(16);

  pin.Input()
	pin.PullUp()
	pin.Detect(rpio.FallEdge) // enable falling edge event detection

  freqPinA.Input()
	freqPinA.PullUp()
	freqPinA.Detect(rpio.RiseEdge) // enable falling edge event detection

  freqPinB.Input()
	freqPinB.PullUp()
	freqPinB.Detect(rpio.RiseEdge) // enable falling edge event detection

	fmt.Println("press a button")

  client := osc.NewClient("127.0.0.1", 57120)

	for i := 0; i < 2; {
		if pin.EdgeDetected() { // check if event occured
			fmt.Println("button pressed")
			i++
		}
    if (freqPinA.EdgeDetected() || freqPinB.EdgeDetected()) {
      blah := rotaryencoder.Read(freqPinA, freqPinB)
      counter += blah
      msg := osc.NewMessage("/frequency")
      msg.Append(int32(counter))
      client.Send(msg)
    }
	}
	pin.Detect(rpio.NoEdge) // disable edge event detection
	freqPinA.Detect(rpio.NoEdge) // disable edge event detection
	freqPinB.Detect(rpio.NoEdge) // disable edge event detection
}
