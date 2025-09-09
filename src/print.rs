use core::arch::asm;

/// SBI console output function
pub fn sbi_putchar(ch: u8) {
    unsafe {
        asm!(
            "ecall",
            in("a6") 0,
            in("a7") 1,
            inout("a0") ch as usize => _,
            out("a1") _
        );
    }
}

/// SBI console input function
pub fn sbi_getchar() -> i32 {
    unsafe {
        let result: i32;
        asm!(
            "ecall",
            in("a6") 0,
            in("a7") 2,
            out("a0") result,
            out("a1") _
        );
        result
    }
}

pub struct Printer;

impl core::fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            sbi_putchar(byte);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = write!($crate::print::Printer, $($arg)*);
    }};
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = writeln!($crate::print::Printer, $($arg)*);
    }};
}

#[macro_export]
macro_rules! print_status {
    ($status:literal, $($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = write!($crate::print::Printer, "[{}] ", $status);
        let _ = writeln!($crate::print::Printer, $($arg)*);
    }};
}

#[macro_export]
macro_rules! print_ok {
    ($($arg:tt)*) => {
        $crate::print_status!("OK", $($arg)*);
    };
}

#[macro_export]
macro_rules! print_fail {
    ($($arg:tt)*) => {
        $crate::print_status!("FAIL", $($arg)*);
    };
}

#[macro_export]
macro_rules! print_info {
    ($($arg:tt)*) => {
        $crate::print_status!("INFO", $($arg)*);
    };
}