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
fn kernel_main(_boot_info: &'static mut BootInfo) -> ! {
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
