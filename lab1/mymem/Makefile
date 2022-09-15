KDIR ?= "/lib/modules/$(shell uname -r)/build"
obj-m += mymem.o

all:
	make -C $(KDIR) M=$(PWD) modules

clean:
	make -C $(KDIR) M=$(PWD) clean

test: test.o
	gcc test.c -o test && ./test

run:
	make all && sudo insmod mymem.ko && make test

