// SPDX-License-Identifier: GPL-2.0

//! Rust mymem module

use kernel::prelude::*;

module! {
    type: RustMyMemDev ,
    name: "rust_mymem",
    author: "Evan Gerritz",
    description: "Rust minimal sample",
    license: "GPL",
}

struct RustMyMemDev {
    message: String,
}

impl kernel::Module for RustMyMemDev {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("mymem module in rust (init)\n");

        Ok( RustMyMemDev {
            message: "on the heap!".try_to_owned()?,
        })
    }
}

impl Drop for RustMyMemDev {
    fn drop(&mut self) {
        pr_info!("My message is {}\n", self.message);
        pr_info!("Rust minimal sample (exit)\n");
    }
}
