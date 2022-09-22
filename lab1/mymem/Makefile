KDIR ?= "/lib/modules/$(shell uname -r)/build"
obj-m += mymem.o

all:
	make -C $(KDIR) M=$(PWD) modules

clean:
	make -C $(KDIR) M=$(PWD) clean

test: test.o
	gcc test.c -o test && ./test

threads: threads.o
	gcc -g3 threads.c -o threads 

run:
	make all && sudo insmod mymem.ko; make threads && ./threads 5 150000

