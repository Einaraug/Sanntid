package main

import (
	. "fmt"
	"runtime"
	"time"
)

var i = 0

func incrementing1() {
	//TODO: increment i 1000000 times
	for k := 0; k < 1000000; k++ {
		i++
	}

}

func decrementing1() {
	//TODO: decrement i 1000000 times
	for k := 0; k < 1000000; k++ {
		i--
	}
}

func main1() {
	// What does GOMAXPROCS do? What happens if you set it to 1?
	//Selects how many CPU cores to be used. When 2, more random numbers appear. When 1 happens more sequentally giving more predictable results
	runtime.GOMAXPROCS(2)

	// TODO: Spawn both functions as goroutines
	go incrementing2()
	go decrementing2()

	// We have no direct way to wait for the completion of a goroutine (without additional synchronization of some sort)
	// We will do it properly with channels soon. For now: Sleep.
	time.Sleep(500 * time.Millisecond)
	Println("The magic number is:", i)
}
