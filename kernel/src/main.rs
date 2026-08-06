#![no_std]
#![no_main]

// bootloader_apiを使うとno_mangleを手書きしなくてよくなるらしい
use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;

// 渡し方を固定するらしいが、何に対してかわかってない
// _startがCPUの最初のjump先。 
// extern "C" でC言語の呼び出し規約(C ABI)に従う
// Rustがfn nameをmangled nameに変換するが、
// このときbootloaderは_startを探すので、no mangleで固定する


entry_point!(kernel_main);
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    
    let framebuffer = boot_info.framebuffer.as_mut().unwrap();
    let info = framebuffer.info();
    let buffer = framebuffer.buffer_mut();

    for chunk in buffer.chunks_exact_mut(info.bytes_per_pixel) {
        chunk[0] = 0x00; //Blue
        chunk[1] = 0xff; //Green
        chunk[2] = 0x00; //Red

    }



    loop {
        core::hint::spin_loop();
    }
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
