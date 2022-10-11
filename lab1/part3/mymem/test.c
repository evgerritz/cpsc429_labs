#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <stdio.h>
#include <assert.h>
#include <stdlib.h>
#include <time.h>


// struct to hold both the avg read and write times in microseconds
// for each number of bytes
struct rw_time
{
    double read;
    double write;
};


// gets time measurements for reads/writes of size num_bytes and
// fills out an rw_time struct
struct rw_time * time_to_read_write(size_t num_bytes) {
    const int TRIALS = 1000;
    clock_t start;
    char i;
    int trial;

    char * buf_to_wrt = malloc(num_bytes);
    char * buf_to_rd = malloc(num_bytes);
    struct rw_time * times = malloc(sizeof(struct rw_time));

    int old;
    int fd = open("/dev/mymem", O_RDWR);
    size_t total_diff_wrt = 0;
    size_t total_diff_rd = 0;
    for (trial = 0; trial < TRIALS; trial++) {
        // generate random buffer, to ensure no caching between trials
        for (i = 0; i < num_bytes; i++) {
             buf_to_wrt[i] = (char) rand() % 255;
        }

        //seek back to beginning
        lseek(fd, 0, SEEK_SET);
        start = clock();
        write(fd, buf_to_wrt, num_bytes);
        old = total_diff_wrt;
        total_diff_wrt += clock() - start;
        assert(total_diff_wrt - old >= 0);

        lseek(fd, 0, SEEK_SET);
        start = clock();
        read(fd, buf_to_rd, num_bytes);
        old = total_diff_rd;
        total_diff_rd += clock() - start;
        assert(total_diff_rd - old >= 0);

        #if DEBUG
        for (i = 0; i < num_bytes; i++) {
            assert(buf_to_wrt[i] == buf_to_rd[i]); 
        }
        #endif
    }
    times->write = total_diff_wrt / (CLOCKS_PER_SEC * TRIALS/1000000);
    times->read = total_diff_rd / (CLOCKS_PER_SEC * TRIALS/1000000);

    // close /dev/mymem and free buffers
    close(fd);
    free(buf_to_wrt);
    free(buf_to_rd);

    return times;
}

#define NUM_SIZES 5

int main () {
    // initialize array of sizes in bytes of the operations
    int sizes[NUM_SIZES] = {1, 64, 1024, 64*1024, 512*1024};
    struct rw_time * times[NUM_SIZES]; 
    int i;
    for (i = 0; i < NUM_SIZES; i++) {
        times[i] = time_to_read_write(sizes[i]);
        printf("%f\t%f\n", times[i]->read, times[i]->write);
    }

    // free the rw_time structs
    for (i = 0; i < NUM_SIZES; i++) {
        free(times[i]);
    }
    return 0;
}
