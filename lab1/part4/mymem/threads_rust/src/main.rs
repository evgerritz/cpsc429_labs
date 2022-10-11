use std::env;
use std::thread;
use std::fs::File;
use std::io;
use std::io::Seek;
use std::io::Read;
use std::io::Write;
use std::sync::Mutex;
use std::sync::Arc;

static INIT_VAL: u64 = 0xDEADBEEF;
const NUM_BYTES: usize = 8;

fn set_counter(f: &mut File, value: u64) -> io::Result<()> {
    f.rewind()?; 
    let n = f.write(&value.to_be_bytes())?; 
    assert!(n == NUM_BYTES);
    Ok(())
}

fn get_counter(f: &mut File) -> io::Result<u64> {
    let mut buf_to_rd: [u8; NUM_BYTES] = [0, 0, 0, 0, 0, 0, 0, 0]; 
    f.rewind()?;
    let n = f.read(&mut buf_to_rd)?;
    assert!(n == NUM_BYTES);
    Ok(u64::from_be_bytes(buf_to_rd))
}


fn create_workers(w: i64, n: i64) -> io::Result<()> {
    let file = File::options().read(true).write(true).open("/dev/mymem")?;
    let file = Arc::new(Mutex::new(file));
    //let counter = Arc::new(Mutex::new(0));

    let mut children = vec![];

    // start w threads
    for _ in 0..w {
        let file = Arc::clone(&file);
        //let counter = Arc::clone(&counter);

        children.push(thread::spawn(move || -> io::Result<()> {
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
    let q: f64 = (actual - expected) / expected;
    q.abs()*100f64 
}

fn get_args() -> (i64, i64) {
    let mut w: i64 = 0;
    let mut n: i64 = 0;
    let args: Vec<_> = env::args().collect();
    if args.len() == 3 {
        w = args[1].parse().unwrap();
        n = args[2].parse().unwrap();
    } else {
        println!("Usage: ./threads_rust w n");
    }
    (w,n)
}

fn avg_counter_after_trials(w: i64, n: i64, num_trials: u64) -> io::Result<u64>{
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
        println!("final: {:?}\tcorrect: {:?}\n", average_counter-INIT_VAL, correct-INIT_VAL);
        println!("percent error: {:?}\n", percent_error(average_counter as f64, correct as f64));
    } else {
        println!("Counter value correct!");
    }
}

fn main () -> io::Result<()>{
    let (w,n) = get_args();

    let num_trials: u64 = 3;
    let average_counter = avg_counter_after_trials(w, n, num_trials)?;

    interpret_results(w, n, average_counter);
    Ok(())
}
