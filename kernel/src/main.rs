#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]


// bootloader_apiを使うとno_mangleを手書きしなくてよくなるらしい
use bootloader_api::{entry_point, BootInfo};
use bootloader_api::info::FrameBufferInfo;
use core::panic::PanicInfo;
use noto_sans_mono_bitmap::{get_raster, FontWeight, RasterHeight};
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use core::fmt::Write;
use spin::Mutex;



// 渡し方を固定するらしいが、何に対してかわかってない
// _startがCPUの最初のjump先。 
// extern "C" でC言語の呼び出し規約(C ABI)に従う
// Rustがfn nameをmangled nameに変換するが、
// このときbootloaderは_startを探すので、no mangleで固定する


entry_point!(kernel_main);


struct Writer {
    buffer: &'static mut [u8],
    info: FrameBufferInfo,
    x: usize,
    y: usize
}

impl Writer {
    fn write_char(&mut self, c:char) {
        if c == '\n' {
            self.x = 10;
            self.y += 16;
            return;
        }

        let glyph = get_raster(c, FontWeight::Regular, RasterHeight::Size16)
            .unwrap_or_else(|| get_raster(' ', FontWeight::Regular, RasterHeight::Size16)
                .unwrap());

        if self.x + glyph.width() > self.info.width {
            self.x = 10;
            self.y += 16;
        }

        for (row, line) in glyph.raster().iter().enumerate() {
            for (col, &intensity) in line.iter().enumerate(){
                let x = self.x + col;
                let y = self.y + row;
                let idx = (y * self.info.stride + x) * self.info.bytes_per_pixel;
                self.buffer[idx] = intensity;
                self.buffer[idx + 1] = intensity;
                self.buffer[idx + 2] = intensity;
            }
        }

        self.x += glyph.width()
    }
} 


impl Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}



static WRITER: Mutex<Option<Writer>> = Mutex::new(None);


macro_rules! println {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        if let Some(writer) = WRITER.lock().as_mut() {
            writeln!(writer, $($arg)*).unwrap();
        }
    }};
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let framebuffer = boot_info.framebuffer.as_mut().unwrap();
    let info = framebuffer.info();
    let buffer = framebuffer.buffer_mut();
    buffer.fill(0);
    
    *WRITER.lock() = Some(Writer { buffer, info, x: 10, y: 10});

    println!("Hello, edu-os from println macro");


    // for chunk in buffer.chunks_exact_mut(info.bytes_per_pixel) {
    //     chunk[0] = 0x00; //Blue
    //     chunk[1] = 0xff; //Green
    //     chunk[2] = 0x00; //Red
    // }
    //
    //

    IDT.load();
    x86_64::instructions::interrupts::int3();
    
    println!("After breakpoint, still alive");

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


lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

extern "x86-interrupt" fn breakpoint_handler(_static_frame: InterruptStackFrame) {}
