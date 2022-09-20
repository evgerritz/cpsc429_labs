#include "threads.h"

#define INIT_VAL 0xabc
#define NUM_BYTES 8

int mem_fd;

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
    uint64_t current_val;
    for (i=0; i<n; i++) {
        current_val = get_counter();
        printf("num: 0x%lX\n", current_val);
        set_counter(current_val+1);
        current_val = get_counter();
        printf("now: 0x%lX\n", current_val);
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

int main ()
{
    mem_fd = open("/dev/mymem", O_RDWR);
    mem_rewind(mem_fd);
    const uint64_t val = INIT_VAL;
    write(mem_fd, &val, NUM_BYTES);

    create_workers(1, 3);

    close(mem_fd);
    return 0;
}
