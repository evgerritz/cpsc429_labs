#include <linux/init.h>
#include <linux/module.h>
MODULE_LICENSE("GPL");

static int mod_init(void) {
	printk(KERN_ALERT "Hello world!\n");
	return 0;
}

static void mod_exit(void) {
	printk(KERN_ALERT "Goodbyte world!\n");
}

module_init(mod_init);
module_exit(mod_exit);
