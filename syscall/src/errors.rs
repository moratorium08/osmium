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
            SyscallError::NoMemorySpace => "No Memory Space",
            SyscallError::InvalidArguments => "Invalid Arguments",
            SyscallError::IllegalFile => "Illegal File",
            SyscallError::NotFound => "Not Found",
            SyscallError::Unknown => "Unknown",
            SyscallError::QueueIsEmpty => "QueueIsEmpty",
            SyscallError::QueueIsFull => "QueueIsFull",
        }
    }
}
