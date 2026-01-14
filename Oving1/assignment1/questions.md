Exercise 1 - Theory questions
-----------------------------

### Concepts

What is the difference between *concurrency* and *parallelism*?
> Concurrency looks like real parallelism, but operations are interleaved in time, rather than simultaneous on different CPU cores

What is the difference between a *race condition* and a *data race*? 
> A data race occurs when two or more threads access the same memory location at the same time and at least one access is a write, and there is no synchronization. This risks that one of the threads get "faulty" values.
A race coondition is a general term referring to, if the programs correctness depends on timing.
 
*Very* roughly - what does a *scheduler* do, and how does it do it?
> A scheduler decides which thread or process runs on the CPU and when.
It selects a task, assigns it time, then switches between tasks based on priority.

### Engineering

Why would we use multiple threads? What kinds of problems do threads solve?
> Higher performance, utilizing more of the CPU. It also allows a program to do "work" while for example waiting for I/O. Increases uptime.

Some languages support "fibers" (sometimes called "green threads") or "coroutines"? What are they, and why would we rather use them over threads?
> Fibers or coroutines are lightweight user-level threads, preferred over OS threads because they are cheaper to create and switch.

Does creating concurrent programs make the programmer's life easier? Harder? Maybe both?
> I think both. Its easier to create high-performance programs, but it is a more demanding process and more problems to navigate. 

What do you think is best - *shared variables* or *message passing*?
> Message passing seems more intuitive to me.


