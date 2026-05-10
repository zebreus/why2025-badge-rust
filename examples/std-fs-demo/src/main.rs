#![allow(unexpected_cfgs)]

use std::{
    env, fs,
    io::{Read, Write},
    path::PathBuf,
};

fn default_path() -> PathBuf {
    #[cfg(target_os = "badgevms")]
    {
        PathBuf::from("FLASH0:[RUST_STD]HELLO.TXT")
    }

    #[cfg(not(target_os = "badgevms"))]
    {
        env::temp_dir().join("why2025-std-fs-demo.txt")
    }
}

fn main() -> std::io::Result<()> {
    let path = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_path);
    println!("writing {}", path.display());

    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        let _ = fs::create_dir_all(parent);
    }

    let mut file = fs::File::create(&path)?;
    file.write_all(b"hello from BadgeVMS std\n")?;
    drop(file);

    let mut text = String::new();
    fs::File::open(&path)?.read_to_string(&mut text)?;
    println!("read back: {text}");

    let metadata = fs::metadata(&path)?;
    println!("len: {}", metadata.len());

    Ok(())
}
