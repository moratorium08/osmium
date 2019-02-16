fn syscall_0(num: u32) -> u32 {
    let result: u32;
    unsafe {
        asm!("
            ecall
        "
        : "={x10}"(result)
        : "{x10}"(num)
        );
    }
    result
}

fn syscall_1(num: u32, a: u32) -> u32 {
    let result: u32;
    unsafe {
        asm!("
            ecall
        "
        : "={x10}"(result)
        : "{x10}"(num), "{x11}"(a)
        );
    }
    result
}

fn syscall_2(num: u32, a: u32, b: u32) -> u32 {
    let result: u32;
    unsafe {
        asm!("
            ecall
        "
        : "={x10}"(result)
        : "{x10}"(num), "{x11}"(a), "{x12}"(b)
        );
    }
    result
}

pub fn sys_write(buf: &[u8], size: usize) -> u32 {
    syscall_2(0, buf.as_ptr() as u32, size as u32)
}

pub fn sys_read(buf: &mut [u8], size: usize) -> u32{
    syscall_2(1, buf.as_ptr() as u32, size as u32)
}

pub fn sys_exit(status: u32) -> ! { 
    syscall_1(2, status);
    loop {} // here does not reach
}

pub fn sys_get_proc_id() -> u32 { 
    syscall_0(4)
}

pub fn sys_yield() -> u32 { 
    syscall_0(5)
}