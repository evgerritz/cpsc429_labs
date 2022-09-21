#include "mymem.h"

#define NAME "mymem"
#define BUF_SIZE 512*1024

MODULE_LICENSE("GPL");

static dev_t first_dev_num = -1;
static struct cdev char_dev;
static struct class * dev_class;

// pointer to buffer in kernel
static char * buffer;
// current location in buffer
static size_t pos;

// fill out file_operations struct with implementations
static struct file_operations fops = {
    .owner = THIS_MODULE,
    .open = mymem_open,
    .read = mymem_read,
    .write = mymem_write,
    .llseek = mymem_llseek,
    .release = mymem_close,
};

// sets the permissions on the created device file to "rw-rw-rw-"
static char * mymem_devnode(struct device *dev, umode_t *mode) {
    if (!mode) return NULL;
    if (dev->devt == first_dev_num) {
        *mode = 0666; 
    }
    return NULL;
}

// initialize the kernel module
static int __init mymem_init(void) {
    int ret_val, i;
    struct device * mymem_device;
    printk(KERN_ALERT "Module mymem init!\n");
    
    // allocate space for the character device
    ret_val = alloc_chrdev_region(&first_dev_num, 0, 1, NAME);
    if (ret_val < 0) return ret_val;

    // create a device class named "chardrv" 
    if (IS_ERR(dev_class = class_create(THIS_MODULE, "chardrv"))) {
        unregister_chrdev_region(first_dev_num, 1);
	    return PTR_ERR(dev_class);
    }
    // link class to permissions setting function
    dev_class->devnode = mymem_devnode;

    // try to create actual device file
    if (IS_ERR(mymem_device = device_create(dev_class, NULL, first_dev_num, NULL, NAME))) {
        class_destroy(dev_class);
        unregister_chrdev_region(first_dev_num, 1);
        return PTR_ERR(mymem_device);
    }

    // initialize the device and add it to the VFS
    cdev_init(&char_dev, &fops);
    ret_val = cdev_add(&char_dev, first_dev_num, 1);

    // allocate buffer in the kernel
    buffer = kmalloc(BUF_SIZE, GFP_USER);    
    if (ret_val < 0 || buffer == NULL) {
        device_destroy(dev_class, first_dev_num) ;
        class_destroy(dev_class);
        unregister_chrdev_region(first_dev_num, 1);
        return (ret_val < 0) ? ret_val : -ENOMEM;
    }

    // clear the buffer
    for (i = 0; i < BUF_SIZE; i++) {
        buffer[i] = '\0'; 
    }
    return 0;
}

static void __exit mymem_exit(void) {
    // deallocate and delete all assoc. data structures
    cdev_del(&char_dev);
    device_destroy(dev_class, first_dev_num);
    class_destroy(dev_class);
    unregister_chrdev_region(first_dev_num, 1);
    kfree(buffer);
    printk("Successfully cleaned up mymem\n");
}

module_init(mymem_init);
module_exit(mymem_exit);


/* Methods */

static int mymem_open(struct inode *inode, struct file *file) {
    printk("Driver: open()\n");
    pos = 0;
    return 0;
}

static int mymem_close(struct inode *inode, struct file *file) {
    printk("Driver: close()\n");
    return 0;
}

// ignore offset
static ssize_t mymem_read(struct file *filp, char __user *user_buffer, size_t length, loff_t *offset) {
    int bytes_read = 0;
    printk("Driver: read()\n");
    
    if (pos + length > BUF_SIZE) {
        return -EINVAL;
    } else {
        // copy_to_user returns number of bytes not copied
        bytes_read = length - copy_to_user(user_buffer,  buffer+pos, length);
    }
    pos += length;
    printk("new pos: %zu\n", pos);
    return bytes_read;
}

static ssize_t mymem_write(struct file *filp, const char *user_buffer, size_t length, loff_t *offset) {
    int bytes_written = 0;
    printk("Driver: write()\n");
    
    if (pos + length > BUF_SIZE) {
        return -EINVAL;
    } else {
        // copy_from_user returns number of bytes not copied
        bytes_written = length - copy_from_user(buffer+pos, user_buffer, length);
    }
    pos += bytes_written;
    printk("new pos: %zu\n", pos);
    return bytes_written;
}

static loff_t mymem_llseek(struct file *filp, loff_t offset, int whence) {
    loff_t newpos;
    printk("Driver: lseek()\n");
    switch(whence) {
         case 0: // SEEK_SET 
     	      newpos = offset;
              break;
         case 1: // SEEK_CUR
              newpos = pos + offset;
              break;
         case 2: // SEEK_END
              newpos = BUF_SIZE + offset;
              break;
         default:
              return -EINVAL;
    } 
    if (newpos < 0) return -EINVAL;
    pos = newpos;
    return newpos;
}
