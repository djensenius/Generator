package main

import (
	"fmt"
	"github.com/wfd3/go-rpio"
  "github.com/hypebeast/go-osc/osc"
	"os"
  "os/exec"
  "rotaryencoder"
  "time"
  //"syscall"
)

var (
  counter = 441
  volume = 0.0
  typeCount = 0
)

func main() {
  if err := rpio.Open(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	defer rpio.Close()

  power := rpio.Pin(3)

  pin := rpio.Pin(16)
  freqPinA := rpio.Pin(20);
  freqPinB := rpio.Pin(21);

  mutePin := rpio.Pin(8)
  volPinA := rpio.Pin(7);
  volPinB := rpio.Pin(12);

  typePin := rpio.Pin(18)
  typePinA := rpio.Pin(24);
  typePinB := rpio.Pin(25);

  power.Input()
	power.PullUp()
	power.Detect(rpio.FallEdge) // enable falling edge event detection

  pin.Input()
	pin.PullUp()
	pin.Detect(rpio.FallEdge) // enable falling edge event detection

  freqPinA.Input()
	freqPinA.PullUp()
	freqPinA.Detect(rpio.RiseEdge) // enable falling edge event detection

  freqPinB.Input()
	freqPinB.PullUp()
	freqPinB.Detect(rpio.RiseEdge) // enable falling edge event detection

  volPinA.Input()
	volPinA.PullUp()
	volPinA.Detect(rpio.RiseEdge) // enable falling edge event detection

  volPinB.Input()
	volPinB.PullUp()
	volPinB.Detect(rpio.RiseEdge) // enable falling edge event detection

  mutePin.Input()
	mutePin.PullUp()
	mutePin.Detect(rpio.FallEdge) // enable falling edge event detection

  typePinA.Input()
	typePinA.PullUp()
	typePinA.Detect(rpio.RiseEdge) // enable falling edge event detection

  typePinB.Input()
	typePinB.PullUp()
	typePinB.Detect(rpio.RiseEdge) // enable falling edge event detection

  typePin.Input()
	typePin.PullUp()
	typePin.Detect(rpio.FallEdge) // enable falling edge event detection

	fmt.Println("press a button")

  client := osc.NewClient("127.0.0.1", 57120)

	for i := 0; i < 2; {
    if power.EdgeDetected() {
      fmt.Println("Going in")
      time.Sleep(800 * time.Millisecond)
      if power.Read() == 1 {
        fmt.Println("Is 1")
        /*
        err := syscall.Reboot(syscall.LINUX_REBOOT_CMD_POWER_OFF)
        if err != nil {
          fmt.Println("Error: ", err)
        }
        */
        cmd := exec.Command("sudo", "/sbin/poweroff")
        er := cmd.Run()
        if er != nil {
          fmt.Println("CMD: ", er)
        }
      }
      fmt.Println("Power Switched", power.Read())
    }

		if pin.EdgeDetected() { // check if event occured
			fmt.Println("button pressed")
			i++
		}

		if mutePin.EdgeDetected() { // check if event occured
			fmt.Println("mute pressed")
		}

		if typePin.EdgeDetected() { // check if event occured
			fmt.Println("type pressed")
		}

    if (freqPinA.EdgeDetected() || freqPinB.EdgeDetected()) {
      blah := rotaryencoder.Read(freqPinA, freqPinB)
      counter += blah
      msg := osc.NewMessage("/frequency")
      msg.Append(int32(counter))
      client.Send(msg)
    }

    if (volPinA.EdgeDetected() || volPinB.EdgeDetected()) {
      blah := rotaryencoder.Read(volPinB, volPinA)
      volume += float64(blah)/100
      msg := osc.NewMessage("/volume")
      msg.Append(float64(volume))
      client.Send(msg)
      fmt.Println("Blah ", blah, volume, msg)
    }

    if (typePinA.EdgeDetected() || typePinB.EdgeDetected()) {
      blah := rotaryencoder.Read(typePinA, typePinB)
      typeCount += blah
      if typeCount > 4 {
        typeCount = 0
      }
      if typeCount < 0 {
        typeCount = 4
      }
      msg := osc.NewMessage("/type")
      msg.Append(int32(typeCount))
      client.Send(msg)
      fmt.Println("Blah ", blah, typeCount, msg)
    }
	}
	pin.Detect(rpio.NoEdge) // disable edge event detection
	freqPinA.Detect(rpio.NoEdge) // disable edge event detection
	freqPinB.Detect(rpio.NoEdge) // disable edge event detection
	volPinA.Detect(rpio.NoEdge) // disable edge event detection
	volPinB.Detect(rpio.NoEdge) // disable edge event detection
}
