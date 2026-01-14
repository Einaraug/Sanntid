package main

import "fmt"
import "time"


func producer(buf chan int){

    for i := 0; i < 10; i++ {
        time.Sleep(100 * time.Millisecond)
        fmt.Printf("[producer]: pushing %d\n", i)
        buf <- i //waits if buffer is full 
    }

}

func consumer(buf chan int){

    time.Sleep(1 * time.Second)
    for {
        i := <-buf //Blocks if no val to be recieved
        fmt.Printf("[consumer]: %d\n", i)
        time.Sleep(50 * time.Millisecond)
    }
}


func main(){
    // TODO: make a bounded buffer
	buf := make(chan int, 4)
    go consumer(buf)
    go producer(buf)
    select {}
}

/*
ERROR CODE GIVEN:
fatal error: all goroutines are asleep - deadlock!

goroutine 1 [select (no cases)]:
main.main()
        /home/matheus/ttk4145/assignments/assignment1/bounded_buffer.go:33 +0x8b

goroutine 18 [chan receive]:
main.consumer(0xc0000b6000)
        /home/matheus/ttk4145/assignments/assignment1/bounded_buffer.go:21 +0x3d
created by main.main in goroutine 1
        /home/matheus/ttk4145/assignments/assignment1/bounded_buffer.go:31 +0x57
exit status 2

########### MEANING ##################
Error msg means all goroutines in the program are blocked. Main is waiting in empty select, consumer is waiting for a routines that no longer sends values
The compiler notices that the program will do no more work and terminates it, instead of it running forever doing nothing.

C compiler would not give thus functionality

*/