use mymem;
use kernel::bindings;
use kernel::prelude::*;
use kernel::{
    file::{self, File},
    //sync::{smutex::Mutex}, Ref, RefBorrow},
    random,
};



/*
static INIT_VAL: u64 = 0xDEADBEEF;
const NUM_BYTES: usize = 8;

fn set_counter(f: &mut File, value: u64) -> Result<()> {
    let n = f.write(&value.to_be_bytes(), 0)?; 
    assert!(n == NUM_BYTES);
    Ok(())
}

fn get_counter(f: &mut File) -> Result<u64> {
    let mut buf_to_rd: [u8; NUM_BYTES] = [0u8; NUM_BYTES];
    let n = f.read(&mut buf_to_rd, 0)?;
    assert!(n == NUM_BYTES);
    Ok(u64::from_be_bytes(buf_to_rd))
}

fn create_workers(w: i64, n: i64) -> Result<()> {
    let file = File::options().read(true).write(true).open("/dev/mymem")?;
    let file = Arc::new(Mutex::new(file));

    let mut children = vec![];

    // start w threads
    for _ in 0..w {
        let file = Arc::clone(&file);

        children.push(thread::spawn(move || -> Result<()> {
            // each thread performs the following atomic action n times
            for _ in 0..n {
                let current_val: u64;
                let mut file = file.lock().unwrap();
                current_val = get_counter(&mut file)?;
                set_counter(&mut file, current_val+1)?;
            }
            Ok(())
        }));
    }

    for child in children {
        // Wait for the thread to finish. Returns a result.
        child.join().unwrap()?;
    }
    Ok(())
}


fn percent_error(actual: f64, expected: f64) -> f64 {
    let mut q: f64 = (actual - expected) / expected;
    if q < 0 {
        q *= -1;
    }
    q*100f64 
}

fn avg_counter_after_trials(w: i64, n: i64, num_trials: u64) -> Result<u64>{
    let mut file = File::options().read(true).write(true).open("/dev/mymem")?;
    let mut counter_total: u64 = 0;
    for _ in 0..num_trials {
        set_counter(&mut file, INIT_VAL)?;
        create_workers(w, n)?;
        counter_total += get_counter(&mut file)?;
    }
    Ok(counter_total / num_trials)
}

fn interpret_results(w: i64, n:i64, average_counter: u64) {
    let correct: u64 = INIT_VAL + (n * w) as u64;
    if average_counter != correct {
        pr_info!("final: {:?}\tcorrect: {:?}\n", average_counter-INIT_VAL, correct-INIT_VAL);
        pr_info!!("percent error: {:?}\n", percent_error(average_counter as f64, correct as f64));
    } else {
        pr_info!("Counter value correct!");
    }
}
*/
/*
unsafe impl AlwaysRefCounted for File {
    fn inc_ref(&self) {
        // SAFETY: The existence of a shared reference means that the refcount is nonzero.
        unsafe { bindings::get_file(self.0.get()) };
    }

    unsafe fn dec_ref(obj: ptr::NonNull<Self>) {
        // SAFETY: The safety requirements guarantee that the refcount is nonzero.
        unsafe { bindings::fput(obj.cast().as_ptr()) }
    }
}
*/



// struct to hold both the avg read and write times in microseconds
// for each number of bytes
struct RWTime {
    read: f64,
    write: f64 
}

// gets time measurements for reads/writes of size num_bytes and
// fills out an rw_time struct
fn time_to_read_write(num_bytes: usize) -> Result<RWTime> {
    let mut buffer: mymem::RustMymem = mymem::RustMymem;

    let mut total_wrt_time: u64 = 0;
    let mut total_rd_time: u64 = 0;
    const TRIALS: u64 = 1000;
    for _ in 0..TRIALS {
        // generate random buffer, to ensure no caching between trials
        let mut buf_to_wrt: Vec<u8> = Vec::try_with_capacity(num_bytes);
        let mut buf_to_rd: Vec<u8> = Vec::try_with_capacity(num_bytes);
        
        random::getrandom(&mut buf_to_wrt[..])?;

        //let start = ProcessTime::try_now().expect("Getting process time failed");
        let n = buffer.write(&buf_to_wrt, 0);
        assert!(n == num_bytes);
        //let cpu_time: Duration = start.try_elapsed().expect("Getting process time failed");
        //println!("{:?}\t{:?}\t{:?}", cpu_time, cpu_time.as_secs(), cpu_time.subsec_micros());
        //total_wrt_time += cpu_time.subsec_micros() as u64;


        //let start2 = ProcessTime::try_now().expect("Getting process time failed");
        let n = buffer.read(&mut buf_to_rd, 0);
        assert!(n == num_bytes);
        //let cpu_time2: Duration = start2.try_elapsed().expect("Getting process time failed");
        //println!("{:?}\t{:?}\t{:?}", cpu_time2, cpu_time2.as_secs(), cpu_time2.subsec_micros());
        //total_rd_time += cpu_time2.subsec_micros() as u64;

        for i in 0..num_bytes {
            assert!(buf_to_wrt[i] == buf_to_rd[i]);
        }
    }

    Ok(RWTime {
        read: total_rd_time as f64 / TRIALS as f64 ,
        write: total_wrt_time as f64 / TRIALS as f64 ,
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
                pr_info!("{:?}\t{:?}", time.read, time.write);
            } else {
                pr_info!("failed!")
            }
        }
    }

    /*
    let run_threads = false;
    if run_threads {

        let num_trials: u64 = 3;
        let average_counter = avg_counter_after_trials(W, N, num_trials)?;

        interpret_results(w, n, average_counter);
    }
    */
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
