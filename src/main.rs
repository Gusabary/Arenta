use std::env;
use std::error::Error;
use std::fs::File;

mod command;
mod manager;
mod task;
mod timeline;

fn print_version() {
    const VERSION: &str = "v1.0.0";
    println!("arenta {VERSION}");
}

fn print_usage() {
    println!("arenta - A daily task management tool with minimal overhead");
    println!("usage: arenta [-hv]");
}

fn arenta_loop() -> Result<(), Box<dyn Error>> {
    let mut lock_file = dirs::home_dir().unwrap();
    lock_file.push(".arenta.lock");

    if let Err(..) = File::options()
        .read(true)
        .write(true)
        .create_new(true)
        .open(lock_file.as_path())
    {
        eprintln!("lock file has been acquired by another process now");
        return Ok(());
    }

    let mut manager = manager::Manager::new();
    manager.start_loop();

    std::fs::remove_file(lock_file.as_path())?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        arenta_loop()
    } else if args.len() == 2 && args[1] == "-v" {
        print_version();
        Ok(())
    } else {
        print_usage();
        Ok(())
    }
}
