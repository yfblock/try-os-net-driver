use core::fmt::{Write, Arguments, Result};
use crate::sbi::console_putchar;

/* 下面的代码跟输出相关，如果不需要输出则直接将相应的代码
 * 注释掉，如果需要输出，则取消注释。在实体代码被注释掉时
 * 相应的代码不会被有效编译，不占内存空间。
 */

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

struct Stdout;

/// 对`Stdout`实现输出的Trait
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> Result {
        let mut buffer = [0u8; 4];
        for c in s.chars() {
            for code_point in c.encode_utf8(&mut buffer).as_bytes().iter() {
                console_putchar(*code_point);
            }
        }
        Ok(())
    }
}

/// 输出函数-u8版
/// 
/// 对传递过来的`u8`数组进行输出
pub fn _puts(args: &[u8]) {
    for i in args {
        console_putchar(*i);
    }
}

/// 输出函数
/// 
/// 对参数进行输出 主要使用在输出相关的宏中 如println
#[no_mangle]
pub fn print(args: Arguments) {
    Stdout.write_fmt(args).unwrap();
}