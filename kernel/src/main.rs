#![no_std]
#![no_main]

// bootloader_apiを使うとno_mangleを手書きしなくてよくなるらしい
use bootloader_api::{entry_point, BootInfo};
use bootloader_api::info::FrameBufferInfo;
use core::panic::PanicInfo;
use noto_sans_mono_bitmap::{get_raster, FontWeight, RasterHeight};

// 渡し方を固定するらしいが、何に対してかわかってない
// _startがCPUの最初のjump先。 
// extern "C" でC言語の呼び出し規約(C ABI)に従う
// Rustがfn nameをmangled nameに変換するが、
// このときbootloaderは_startを探すので、no mangleで固定する


entry_point!(kernel_main);



fn draw_char(buffer: &mut [u8], info: FrameBufferInfo, x_off: usize, y_off: usize, c: char) -> usize {
    let glyph = get_raster(c, FontWeight::Regular, RasterHeight::Size16)
        .unwrap_or_else(|| get_raster(' ', FontWeight::Regular, RasterHeight::Size16)
            .unwrap());

    for (row, line) in glyph.raster().iter().enumerate() {
        for (col, &intensity) in line.iter().enumerate(){
            let x = x_off + col;
            let y = y_off + row;
            let idx = (y * info.stride + x) * info.bytes_per_pixel;
            buffer[idx] = intensity;
            buffer[idx + 1] = intensity;
            buffer[idx + 2] = intensity;
        }
    }

    glyph.width()
}


fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let framebuffer = boot_info.framebuffer.as_mut().unwrap();
    let info = framebuffer.info();
    let buffer = framebuffer.buffer_mut();
    buffer.fill(0);
    
    let mut x = 10;
    for c in "Hello, edu-os!".chars() {
        x += draw_char(buffer, info, x, 10, c)
    }

    // for chunk in buffer.chunks_exact_mut(info.bytes_per_pixel) {
    //     chunk[0] = 0x00; //Blue
    //     chunk[1] = 0xff; //Green
    //     chunk[2] = 0x00; //Red
    // }
    //


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
