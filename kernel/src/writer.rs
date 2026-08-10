use bootloader_api::info::FrameBufferInfo;
use noto_sans_mono_bitmap::{get_raster, FontWeight, RasterHeight};
use core::fmt::Write;
use spin::Mutex;



// 渡し方を固定するらしいが、何に対してかわかってない
// _startがCPUの最初のjump先。 
// extern "C" でC言語の呼び出し規約(C ABI)に従う
// Rustがfn nameをmangled nameに変換するが、
// このときbootloaderは_startを探すので、no mangleで固定する




pub struct Writer {
    buffer: &'static mut [u8],
    info: FrameBufferInfo,
    x: usize,
    y: usize
}

impl Writer {
    pub fn new(buffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        Self { buffer, info, x: 10, y: 10 }
    }
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



pub static WRITER: Mutex<Option<Writer>> = Mutex::new(None);

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        if let Some(writer) = $crate::writer::WRITER.lock().as_mut() {
            writeln!(writer, $($arg)*).unwrap();
        }
    }};
}

