#include "threads.h"

#define INIT_VAL 0xdeadbeef
#define NUM_BYTES 8

int mem_fd;
pthread_mutex_t mymem_lock;

// reset position in memory buffer to 0
void mem_rewind() {
    (void) lseek(mem_fd, 0, SEEK_SET);
}

// utility function for setting the counter to value
void set_counter(const uint64_t value) {
    mem_rewind();
    (void) write(mem_fd, &value, NUM_BYTES);
}

// utility function for returning the current value of the counter
uint64_t get_counter() {
    uint64_t value;
    mem_rewind();
    (void) read(mem_fd, &value, NUM_BYTES);
    return value;
}

//long num_missed = 0; unused because reference in do_work is commented out

// perform *n_ptr iterations of getting counter value, incrementing, writing
void * do_work(void * n_ptr) {
    int n = *((int *) n_ptr);
    int i;
    uint64_t current_val, read_val;
    for (i=0; i<n; i++) {
        pthread_mutex_lock(&mymem_lock);
        current_val = get_counter();
        set_counter(current_val+1);
        /*  --- this code checks whether the current value is what it is supposed to be
         *      but n=150000 is slow, so I wanted to get rid of unnecessary work
        read_val = get_counter();
        if (read_val > current_val+1) {
            num_missed++;
        }
        //printf("%lu -> %lu\n", current_val-INIT_VAL, read_val-INIT_VAL);
        */
        pthread_mutex_unlock(&mymem_lock);
    }
}

// creates w worker threads and has them perform n iterations of work
void create_workers(int w, int n) {
    pthread_t workers[w];	
    int i;
    // create the workers and make them do_work
    for (i=0; i<w; i++) {
        (void) pthread_create(&workers[i], NULL, do_work, &n);
    }
    
    // wait for all the threads to finish before returning
    for (i=0; i<w; i++) {
        pthread_join(workers[i], NULL);
    }
}

// calculate the percent error between the actual and expected counter values
double percent_error(double actual, double expected) {
    double q = (actual - expected) / expected;
    return fabs(q)*100;//% 
}

// input format: ./threads w n
int main (int argc, char **argv) {
    // read in the arguments
    int w = 0;
    int n = 0;
    sscanf(argv[1], "%d", &w);
    sscanf(argv[2], "%d", &n);

    mem_fd = open("/dev/mymem", O_RDWR);

    const int TRIALS = 3;
    uint64_t counter_total = 0;
    uint64_t average_counter = 0;

    // get the final counter value on TRIALS trials, take the average
    for (int i = 0; i < TRIALS; i++) {
        // reset the counter
        set_counter(INIT_VAL);

        // create and start the child threads
        create_workers(w, n);
        // by now all child threads have terminated
        counter_total += get_counter();
    }
    average_counter = counter_total/TRIALS;
    
    // interpret and print results
    uint64_t correct = INIT_VAL + (n * w);
    if (average_counter != correct) {
        printf("final: %lu\tcorrect: %lu\n", average_counter-INIT_VAL, correct-INIT_VAL);
        printf("percent error: %lf\n", percent_error(average_counter-INIT_VAL, correct-INIT_VAL));
    } else {
        printf("Counter value correct!\n");
    }

    close(mem_fd);

    return 0;
}
