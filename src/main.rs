use std::path::PathBuf;
use std::process::Command;


// bootloaderが返すerror typeがanyhow::Resultだからいったんそれを受け取る
fn main() -> anyhow::Result<()> {
    // まずcargo buildでELFを作る
    let status = Command::new("cargo")
        .args(["build", "--target", "x86_64-unknown-none"])
        .current_dir("kernel")
        .status()?;
    anyhow::ensure!(status.success(), "kernel build failed");

    // BiosBootでELFをdisk image化
    let kernel_elf = PathBuf::from("kernel/target/x86_64-unknown-none/debug/edu-os");
    let image = PathBuf::from("target/disk.img");
    
    bootloader::BiosBoot::new(&kernel_elf).create_disk_image(&image)?;
    
    // disk imageをQEMUにぶち込む
    let status = Command::new("qemu-system-x86_64")
        .arg("-drive")
        .arg(format!("format=raw,file={}", image.display()))
        .status()?;
    anyhow::ensure!(status.success(), "qemu exited with error");

    Ok(())
}
