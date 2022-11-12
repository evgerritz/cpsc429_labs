use kernel::prelude::*;
use kernel::sync::smutex::Mutex;

use kernel::bindings;

module! {
    type: RustCamera,
    name: "rust_camera",
    author: "Guojun Chen",
    description: "A simple module that reads camera input.",
    license: "GPL",
}

struct RustCamera {}

impl kernel::Module for RustCamera {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("RustCamera (init)\n");
        // make RustCamera a miscdev as you have done in A1P4

        Ok(RustCamera {})
    }
}

impl Drop for RustCamera {
    fn drop(&mut self) {
        pr_info!("RustCamera (exit)\n");
    }
}
