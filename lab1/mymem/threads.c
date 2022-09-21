#include "threads.h"

#define INIT_VAL 0xdeadbeef
#define NUM_BYTES 8

int mem_fd;
long num_missed = 0;

void mem_rewind() {
    lseek(mem_fd, 0, SEEK_SET);
}

void set_counter(const uint64_t value) {
    mem_rewind();
    write(mem_fd, &value, NUM_BYTES);
}

uint64_t get_counter() {
    uint64_t value;
    mem_rewind();
    read(mem_fd, &value, NUM_BYTES);
    return value;
}

void * do_work(void * n_ptr) {
    int n = *((int *) n_ptr);
    int i;
    uint64_t current_val, read_val;
    for (i=0; i<n; i++) {
        current_val = get_counter();
        set_counter(current_val+1);
        read_val = get_counter();
        if (read_val > current_val+1) {
            num_missed++;
        }
        //printf("%lu -> %lu\n", current_val-INIT_VAL, read_val-INIT_VAL);
    }
}

void create_workers(int w, int n) {
    pthread_t workers[w];	
    int irets[w];
    int i;
    for (i=0; i<w; i++) {
        irets[i] = pthread_create(&workers[i], NULL, do_work, &n);
    }
    
    for (i=0; i<w; i++) {
        pthread_join(workers[i], NULL);
    }
}

int main (int argc, char **argv)
{
    // input format: ./threads w n
    int w = 0;
    int n = 0;
    sscanf(argv[1], "%d", &w);
    sscanf(argv[2], "%d", &n);

    mem_fd = open("/dev/mymem", O_RDWR);
    set_counter(INIT_VAL);

    create_workers(w, n);

    uint64_t correct = INIT_VAL + (n * w);
    if (get_counter() != correct) {
        printf("final: %lu\tcorrect: %lu\n", get_counter()-INIT_VAL, correct-INIT_VAL);
    }
    printf("number missed: %lu\n", num_missed);

    close(mem_fd);

    return 0;
}
