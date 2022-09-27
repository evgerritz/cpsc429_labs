extern crate rand;

use rand::Rng;
use std::fs::File;
use std::io;
use std::io::Seek;
use std::io::Read;
use std::io::Write;

use std::time::Duration;
use cpu_time::ProcessTime;

// struct to hold both the avg read and write times in microseconds
// for each number of bytes
struct RWTime {
    read: f64,
    write: f64 
}

fn duration_to_secs(duration: Duration) -> f64 {
    duration.as_secs() as f64 + (duration.subsec_nanos() as f64) * 1e-9
}

// gets time measurements for reads/writes of size num_bytes and
// fills out an rw_time struct
fn time_to_read_write(num_bytes: usize) -> io::Result<RWTime> {
    let mut f = File::options().read(true).write(true).open("/dev/mymem")?;

    let mut total_diff_wrt = Duration::new(0, 0);
    let mut total_diff_rd = Duration::new(0, 0);
    const TRIALS: u64 = 100;
    for _ in 0..TRIALS {
        // generate random buffer, to ensure no caching between trials
        let mut buf_to_wrt: Vec<u8> = vec![0; num_bytes];
        let mut buf_to_rd: Vec<u8> = vec![0; num_bytes];

        for i in 0..num_bytes {
             buf_to_wrt[i] = rand::thread_rng().gen();
        }

        //seek back to beginning
        f.rewind()?;
        let start = ProcessTime::try_now().expect("Getting process time failed");
        let n = f.write(&buf_to_wrt[0..])?;
        assert!(n == num_bytes);
        let cpu_time: Duration = start.try_elapsed().expect("Getting process time failed");
        total_diff_wrt += cpu_time;


        f.rewind()?;
        let start = ProcessTime::try_now().expect("Getting process time failed");
        let n = f.read(&mut buf_to_rd)?;
        assert!(n == num_bytes);
        let cpu_time: Duration = start.try_elapsed().expect("Getting process time failed");
        total_diff_rd += cpu_time;
        for i in 0..num_bytes {
            assert!(buf_to_wrt[i] == buf_to_rd[i]);
        }
    }

    Ok(RWTime {
        read: duration_to_secs(total_diff_rd)*1e6 / TRIALS as f64,
        write: duration_to_secs(total_diff_wrt)*1e6 / TRIALS as f64,
    })
}

fn main () {
    // initialize array of sizes in bytes of the operations
    const NUM_SIZES: usize = 5;
    const SIZES: [usize; NUM_SIZES] = [1, 64, 1024, 64*1024, 512*1024];
    for i in 0..NUM_SIZES {
        if let Ok(time) = time_to_read_write(SIZES[i]) {
            println!("{:.2}\t{:.2}", time.read, time.write);
        } else {
            println!("failed!")
        }
    }
}
