# Part 5: Reflecting
## How do the mechanisms differ?
Condition variables require manual locking and while-loops. Ada protected objects use declarative guards that the runtime evaluates automatically. Ada is higher-level and less error-prone.

## Which solutions are you most confident in?
Ada protected objects. The code is minimal and reads like English. Less code means fewer bugs.

## Extending to N priorities?
Priority queue solutions scale effortlessly — just change the priority integer. Ada would need N separate entries with increasingly long guards. Go nested selects cannot scale at all.

## Is getValue for semaphores appropriate?
No. Checking the value and then acting on it is a race condition. Semaphores are meant for atomic operations only.

## Which mechanism do you prefer?
Ada for simplicity when priorities are fixed. Priority queues for real-world use since they generalize to N priorities without code changes.
