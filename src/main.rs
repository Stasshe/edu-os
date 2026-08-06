#![no_std]
#![no_main]

use core::panic::PanicInfo;

// 渡し方を固定するらしいが、何に対してかわかってない
// _startがCPUの最初のjump先。 
// extern "C" でC言語の呼び出し規約(C ABI)に従う
// Rustがfn nameをmangled nameに変換するが、
// このときbootloaderは_startを探すので、no mangleで固定する

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
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
