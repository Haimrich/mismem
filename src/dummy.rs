use std::thread::sleep;
use std::time::Duration;

fn main() {
    let mut old_var : u64 = 0;
    let mut var : u64 = 0;
    
    loop {
        var += 1;
        
        let msg = if old_var + 1 == var {
            "👍 Increased Counter by one: "
        } else {
            "🚨 \x1b[1m\x1b[31mThe counter has been altered! New value: "
        };
        
        println!("{}{}\x1b[0m", msg , var);

        old_var = var;
        sleep(Duration::from_secs(5));
    }
}