// Compile with `gcc foo.c -Wall -std=gnu99 -lpthread`, or use the makefile
// The executable will be named `foo` if you use the makefile, or `a.out` if you use gcc directly

#include <pthread.h>
#include <stdio.h>

int i = 0;
pthread_mutex_t m = PTHREAD_MUTEX_INITIALIZER;

// Note the return type: void*
void* incrementingThreadFunction(){
    // TODO: increment i 1_000_000 times
    for(int k = 0; k < 1000000; k++){
        pthread_mutex_lock(&m); //Lock variable
        i++;
        pthread_mutex_unlock(&m); //Unlock variable

        if (k % 1000 == 0) printf("inc is yielding \n"); sched_yield(); //Force interleaving
    }
    return NULL;
}

void* decrementingThreadFunction(){
    // TODO: decrement i 1_000_000 times
    for(int k = 0; k < 1000005; k++){
        pthread_mutex_lock(&m);
        i--;
        pthread_mutex_unlock(&m);

        if (k % 1000 == 0) sched_yield(); //Force interleaving
    }
    return NULL;
}


int main(){
    // TODO: 
    // start the two functions as their own threads using `pthread_create`
    // Hint: search the web! Maybe try "pthread_create example"?
    pthread_t thread_1;
    if(pthread_create(&thread_1, NULL, incrementingThreadFunction, NULL) != 0){
        perror("ERROR creating thread.");
    }
    
    pthread_t thread_2;
    if(pthread_create(&thread_2, NULL, decrementingThreadFunction, NULL) != 0){
        perror("ERROR creating thread.");
    }
    
    // TODO:
    // wait for the two threads to be done before printing the final result
    // Hint: Use `pthread_join`    
    pthread_join(thread_1, NULL);
    pthread_join(thread_2, NULL);
    
    printf("The magic number is: %d\n", i);
    return 0;
}

//Use a mutex to protect shared data; use a semaphore to count or signal resources.