//! test program for mymem rust module written as kernel module

use mymem;
use kernel::bindings;
use kernel::prelude::*;
use kernel::{
    sync::{smutex::Mutex, Ref}
    random,
    task::Task,
};



static INIT_VAL: u64 = 0xDEADBEEF;
const NUM_BYTES: usize = 8;

fn set_counter(buf: &mut mymem::RustMymem, value: u64) -> Result<()> {
    let n = buf.write(&value.to_be_bytes(), 0); 
    assert!(n == NUM_BYTES);
    Ok(())
}

fn get_counter(buf: &mut mymem::RustMymem) -> Result<u64> {
    let mut buf_to_rd: [u8; NUM_BYTES] = [0u8; NUM_BYTES];
    let n = buf.read(&mut buf_to_rd, 0);
    assert!(n == NUM_BYTES);
    Ok(u64::from_be_bytes(buf_to_rd))
}

fn create_workers(w: i64, n: i64) -> Result<()> {
    let mut buffer: mymem::RustMymem = mymem::RustMymem;
    let buffer = Ref::try_new(Mutex::new(buffer).as_ptr());

    let mut children = Vec::new();

    // start w threads
    for _ in 0..W {
    let buffer = BUFFER.clone();
    children.try_push(Task::spawn(fmt!(""), move || {
        for _ in 0..N {
            let current_val: u64;
            let buffer = &mut buffer.lock();
            current_val = get_counter(&mut buffer).unwrap();
            set_counter(&mut buffer, current_val+1).unwrap();
        }
    })?)?;


    /*for child in children {
        // Wait for the thread to finish. Returns a result.
        child.join().unwrap()?;
    }*/
    Ok(())
}


fn avg_counter_after_trials(w: i64, n: i64, num_trials: u64) -> Result<u64>{
    let mut counter_total: u64 = 0;
    for _ in 0..num_trials {
        let buffer = &mut BUFFER.lock();
        set_counter(&mut buffer, INIT_VAL)?;
        create_workers()?;
        counter_total += get_counter(&mut buffer)?;
    }
    Ok(counter_total / num_trials)
}

fn interpret_results(w: i64, n:i64, average_counter: u64) {
    let correct: u64 = INIT_VAL + (n * w) as u64;
    if average_counter != correct {
        pr_info!("final: {:?}\tcorrect: {:?}\n", average_counter-INIT_VAL, correct-INIT_VAL);
    } else {
        pr_info!("Counter value correct!");
    }
}



// struct to hold both the avg read and write times in microseconds
// for each number of bytes
struct RWTime {
    read: u64,
    write: u64 
}

// gets time measurements for reads/writes of size num_bytes and
// fills out an rw_time struct
fn time_to_read_write(num_bytes: usize) -> Result<RWTime> {
    let mut buffer: mymem::RustMymem = mymem::RustMymem;

    let mut total_wrt_time: u64 = 0;
    let mut total_rd_time: u64 = 0;
    const TRIALS: u64 = 1024;
    for _ in 0..TRIALS {
        // generate random buffer, to ensure no caching between trials
        let mut buf_to_wrt: Vec<u8> = Vec::try_with_capacity(num_bytes)?;
        let mut buf_to_rd: Vec<u8> = Vec::try_with_capacity(num_bytes)?;
        buf_to_wrt.try_resize(num_bytes, 0)?;
        buf_to_rd.try_resize(num_bytes, 0)?;
        
        random::getrandom(&mut buf_to_wrt)?;

        let mut start = bindings::timespec64 {tv_sec: 0, tv_nsec: 0};
        let mut end = bindings::timespec64 {tv_sec: 0, tv_nsec: 0};
        unsafe { bindings::ktime_get_ts64(&mut start); }
        let n = buffer.write(&buf_to_wrt, 0);
        unsafe { bindings::ktime_get_ts64(&mut end); }
        assert!(n == num_bytes);
        assert!(end.tv_sec - start.tv_sec == 0);
        total_wrt_time += (end.tv_nsec - start.tv_nsec) as u64;

        unsafe { bindings::ktime_get_ts64(&mut start); }
        let n = buffer.read(&mut buf_to_rd, 0);
        unsafe { bindings::ktime_get_ts64(&mut end); }
        assert!(n == num_bytes);
        assert!(end.tv_sec - start.tv_sec == 0);
        total_rd_time += (end.tv_nsec - start.tv_nsec) as u64;

        for i in 0..num_bytes {
            assert!(buf_to_wrt[i] == buf_to_rd[i]);
        }
    }

    Ok(RWTime {
        read: total_rd_time >> 10,
        write: total_wrt_time >> 10
    })
}

const W: i64 = 10;
const N: i64 = 10;

// while we are loading this code as a module, it really is just a program;
// this main function makes that idea explicit.
fn main () -> Result<()>{
    let run_timing = true;
    if run_timing {
        // initialize array of sizes in bytes of the operations
        const NUM_SIZES: usize = 5;
        const SIZES: [usize; NUM_SIZES] = [1, 64, 1024, 64*1024, 512*1024];
        for i in 0..NUM_SIZES {
            if let Ok(time) = time_to_read_write(SIZES[i]) {
                pr_info!("{:?}\t{:?}\t{:?}", SIZES[i], time.read, time.write);
            } else {
                pr_info!("failed!")
            }
        }
    }

    let run_threads = false;
    if run_threads {

        let num_trials: u64 = 3;
        let average_counter = avg_counter_after_trials(W, N, num_trials)?;

        interpret_results(W, N, average_counter);
    }
    Ok(())
}


module! {
    type: MymemTest,
    name: "mymem_test",
    author: "Evan Gerritz",
    description: "mymem_test module in Rust",
    license: "GPL",
}

struct MymemTest;

impl kernel::Module for MymemTest {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("mymem_test (init)\n");
        main()?;
        Ok(MymemTest)
    }
}

impl Drop for MymemTest {
    fn drop(&mut self) {
        pr_info!("mymem_test (exit)\n");
    }
}
