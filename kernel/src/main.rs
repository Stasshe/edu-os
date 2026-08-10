#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]


mod writer;
mod gdt;

// bootloader_apiを使うとno_mangleを手書きしなくてよくなるらしい
use bootloader_api::{entry_point, BootInfo};
// use bootloader_api::info::FrameBufferInfo;
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
// use x86_64::structures::idt::PageFaultErrorCode;



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
    buffer.fill(0);
    
    *writer::WRITER.lock() = Some(writer::Writer::new(buffer, info));
    println!("Hello, edu-os from println macro");


    // for chunk in buffer.chunks_exact_mut(info.bytes_per_pixel) {
    //     chunk[0] = 0x00; //Blue
    //     chunk[1] = 0xff; //Green
    //     chunk[2] = 0x00; //Red
    // }
    //
    //

    gdt::init();
    IDT.load();
    // x86_64::instructions::interrupts::int3();
    //
    // println!("After breakpoint, still alive");
    
    // unsafe {
    //     *(0xdeabdeef as *mut u8) = 42;
    // }

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
        unsafe {
            idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

extern "x86-interrupt" fn breakpoint_handler(_static_frame: InterruptStackFrame) {}

extern "x86-interrupt" fn double_fault_handler (
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    println!("DOUBLE FAULT\n{:#?}", stack_frame);
    loop {
        core::hint::spin_loop();
    }
}

