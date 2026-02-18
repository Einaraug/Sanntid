package main

import (
    . "fmt"
    "runtime"
)



/*
Using shared variable synchronization is possible, but not the idiomatic approach in Go. You should instead create a "server" that is responsible for its own data, select{}s messages, and perform different actions on its data when it receives a corresponding message.

In this case, the data is the integer i, and the three actions it can perform are increment, decrement, and read (or "get"). Two other goroutines should send the increment and decrement requests to the number-server, and main should read out the final value after these two goroutines are done.

Before attempting to do the exercise, it is recommended to have a look at the following chapters of the interactive go tutorial:
*/
func incrementing2(inc chan int, done chan int) {
    //TODO: increment i 1000000 times
	for k := 0; k < 1000005; k++{
		inc <- 1
	}
	done <- 1

}

func decrementing2(dec chan int, done chan int) {
    //TODO: decrement i 1000000 times
	for k := 0; k < 1000000; k++{
		dec <- 1
	} 
	done <- 1
}


func i_server(inc chan int, dec chan int, get chan int, quit chan int, data chan int){
	i := 0
	for{
		select{
		case <- inc:
			i++
		case <- dec:
			i--
		case <- get:
			data <- i
		case <- quit:
			return
		}
	}
}


func main2() {
    // What does GOMAXPROCS do? What happens if you set it to 1?

    runtime.GOMAXPROCS(2)    
	
    // TODO: Spawn both functions as goroutines
	inc := make(chan int)
	dec := make(chan int)
	get := make(chan int)
	done := make(chan int)
	quit := make(chan int)
	data := make(chan int)
	go i_server(inc, dec, get, quit, data)

	//Start sub_routines
	go incrementing2(inc, done)
	go decrementing2(dec, done)

	//Join sub_routines
	<- done
	<- done

	get <- 1
	final_value := <-data

    Println("The magic number with channels is:", final_value)
	
	// Stop server
	quit <- 1
}