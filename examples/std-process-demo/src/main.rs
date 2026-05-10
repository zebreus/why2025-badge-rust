#![allow(unexpected_cfgs)]

use std::{env, process::Command};

fn main() -> std::io::Result<()> {
    if env::args().any(|arg| arg == "--child") {
        println!("child argv:");
        for (index, argument) in env::args().enumerate() {
            println!("  {index}: {argument}");
        }
        return Ok(());
    }

    let child_path = env::args().nth(1).unwrap_or_else(default_child_path);
    println!("spawning child path: {child_path}");

    let status = Command::new(&child_path).arg("--child").status()?;
    println!("child status: {status}");
    Ok(())
}

fn default_child_path() -> String {
    #[cfg(target_os = "badgevms")]
    {
        "APP:std-process-demo".to_owned()
    }

    #[cfg(not(target_os = "badgevms"))]
    {
        env::current_exe()
            .expect("host current_exe should be available")
            .display()
            .to_string()
    }
}
