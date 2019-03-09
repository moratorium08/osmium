use core::fmt;

#[derive(Debug, Copy, Clone)]
pub enum SyscallError {
    InvalidSyscallNumber,
    InternalError,
    TooManyProcess,
    NoMemorySpace,
    InvalidArguments,
    IllegalFile,
    NotFound,
    Unknown,
    QueueIsEmpty,
    QueueIsFull,
    PermissionDenied,
    InvalidAlignment,
}

impl SyscallError {
    pub fn to_syscall_result(&self) -> i32 {
        match self {
            SyscallError::InvalidSyscallNumber => -1,
            SyscallError::InternalError => -2,
            SyscallError::TooManyProcess => -3,
            SyscallError::NoMemorySpace => -4,
            SyscallError::InvalidArguments => -5,
            SyscallError::IllegalFile => -6,
            SyscallError::NotFound => -7,
            SyscallError::Unknown => -8,
            SyscallError::QueueIsEmpty => -9,
            SyscallError::QueueIsFull => -10,
            SyscallError::PermissionDenied => -11,
            SyscallError::InvalidAlignment => -12,
        }
    }

    pub fn from_syscall_result(v: i32) -> SyscallError {
        match v {
            -1 => SyscallError::InvalidSyscallNumber,
            -2 => SyscallError::InternalError,
            -3 => SyscallError::TooManyProcess,
            -4 => SyscallError::NoMemorySpace,
            -5 => SyscallError::InvalidArguments,
            -6 => SyscallError::IllegalFile,
            -7 => SyscallError::NotFound,
            -8 => SyscallError::Unknown,
            -9 => SyscallError::QueueIsEmpty,
            -10 => SyscallError::QueueIsFull,
            -11 => SyscallError::PermissionDenied,
            -12 => SyscallError::InvalidAlignment,
            _ => SyscallError::Unknown,
        }
    }
}

impl fmt::Display for SyscallError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl SyscallError {
    pub fn to_str(&self) -> &'static str {
        match self {
            SyscallError::InvalidSyscallNumber => "Invalid Syscall Number",
            SyscallError::InternalError => "Internal error",
            SyscallError::TooManyProcess => "Too many process",
            SyscallError::NoMemorySpace => "No memory space",
            SyscallError::InvalidArguments => "Invalid arguments",
            SyscallError::IllegalFile => "Illegal file",
            SyscallError::NotFound => "Not Found",
            SyscallError::Unknown => "Unknown",
            SyscallError::QueueIsEmpty => "Queue is empty",
            SyscallError::QueueIsFull => "Queue is full",
            SyscallError::PermissionDenied => "Permission denied",
            SyscallError::InvalidAlignment => "Invalid alignment",
        }
    }
}
