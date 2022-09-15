#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <stdio.h>
#include <assert.h>
#include <stdlib.h>
#include <time.h>

int basic() {
    int fd = open("/dev/mymem", O_RDWR); 
    write(fd, "test123", 7);
    lseek(fd, 0, SEEK_SET);
    char buf[512];
    read(fd, buf, 7);
    printf("%s\n", buf);    
    close(fd);
    return 0;
}

struct rw_time
{
    double read;
    double write;
};


struct rw_time * time_to_read_write(size_t num_bytes) {
    char * buf_to_wrt = malloc(num_bytes);
    char * buf_to_rd = malloc(num_bytes);
    char i;
    int trial;
    const int TRIALS = 1000;
    clock_t start;
    struct rw_time * times = malloc(sizeof(struct rw_time));

    int old;
    int fd = open("/dev/mymem", O_RDWR);
    size_t total_diff_wrt = 0;
    size_t total_diff_rd = 0;
    for (trial = 0; trial < TRIALS; trial++) {
	for (i = 0; i < num_bytes; i++) {
	     // generate random buffer
	     buf_to_wrt[i] = (char) rand() % 255;
	}

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

    close(fd);
    free(buf_to_wrt);
    free(buf_to_rd);

    return times;
}

#define NUM_SIZES 5

int main () {
    int sizes[NUM_SIZES] = {1, 64, 1024, 64*1024, 512*1024};
    struct rw_time * times[NUM_SIZES]; 
    int i;
    for (i = 0; i < NUM_SIZES; i++) {
        times[i] = time_to_read_write(sizes[i]);
	printf("%d\t:%f\t%f\n", sizes[i], times[i]->read, times[i]->write);
    }
    for (i = 0; i < NUM_SIZES; i++) {
	free(times[i]);
    }
    return 0;
}
