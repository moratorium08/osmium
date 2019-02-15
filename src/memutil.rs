pub unsafe fn memset(buf: *mut u8, byte: u8, size: usize) {
    let mut buf = buf;
    for _ in 0..size {
        *buf = byte;
        buf = (buf as usize + 1) as *mut u8;
    }
}

pub unsafe fn memcpy(to: *mut u8, from: &[u8], size: usize) {
    let mut buf = to;
    for i in 0..size {
        *buf = from[i];
        buf = (buf as usize + 1) as *mut u8;
    }
}
