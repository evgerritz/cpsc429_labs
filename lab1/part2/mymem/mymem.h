#include <linux/kernel.h>
#include <linux/version.h>
#include <linux/module.h>
#include <linux/init.h>
#include <linux/fs.h>
#include <linux/slab.h>
#include <linux/kdev_t.h>
#include <linux/errno.h>
#include <linux/device.h>
#include <linux/types.h>
#include <linux/cdev.h>

static char * mymem_devnode(struct device *, umode_t *);
static int mymem_init(void);
static void mymem_exit(void);
static int mymem_open(struct inode *, struct file *);
static int mymem_close(struct inode *, struct file *);
static ssize_t mymem_read(struct file *, char *, size_t, loff_t *);
static ssize_t mymem_write(struct file *, const char *, size_t, loff_t *);
static loff_t mymem_llseek(struct file *, loff_t, int);

