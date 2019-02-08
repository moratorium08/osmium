#[derive(Clone, Copy, Debug)]
pub enum Interruption {
    UserSoftware,
    SupervisorSoftware,
    MachineSoftware,
    UserTimer,
    SupervisorTimer,
    MachineTimer,
    UserExternal,
    SupervisorExternal,
    MachineExternal,
}

impl Interruption {
    pub fn to_u32(self) -> u32 {
        match self {
            Interruption::UserSoftware => 0,
            Interruption::SupervisorSoftware => 1,
            Interruption::MachineSoftware => 3,
            Interruption::UserTimer => 4,
            Interruption::SupervisorTimer => 5,
            Interruption::MachineTimer => 7,
            Interruption::UserExternal => 8,
            Interruption::SupervisorExternal => 9,
            Interruption::MachineExternal => 11,
        }
    }
    pub fn from_u32(x: u32) -> Option<Interruption> {
        match x {
            0 => Some(Interruption::UserSoftware),
            1 => Some(Interruption::SupervisorSoftware),
            3 => Some(Interruption::MachineSoftware),
            4 => Some(Interruption::UserTimer),
            5 => Some(Interruption::SupervisorTimer),
            7 => Some(Interruption::MachineTimer),
            8 => Some(Interruption::UserExternal),
            9 => Some(Interruption::SupervisorExternal),
            11 => Some(Interruption::MachineExternal),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Exception {
    InstructionAddressMisaligned,
    InstructionAccessFault,
    IllegalInstruction,
    Breakpoint,
    LoadAccessMisaligned,
    LoadAccessFault,
    StoreAddressMisalinged,
    StoreAccessFault,
    EnvironmentCallU,
    EnvironmentCallS,
    EnvironmentCallM,
    InstructionPageFault,
    LoadPageFault,
    StorePageFault,
}

impl Exception {
    pub fn to_u32(self) -> u32 {
        match self {
            Exception::InstructionAddressMisaligned => 0,
            Exception::InstructionAccessFault => 1,
            Exception::IllegalInstruction => 2,
            Exception::Breakpoint => 3,
            Exception::LoadAccessMisaligned => 4,
            Exception::LoadAccessFault => 5,
            Exception::StoreAddressMisalinged => 6,
            Exception::StoreAccessFault => 7,
            Exception::EnvironmentCallU => 8,
            Exception::EnvironmentCallS => 9,
            Exception::EnvironmentCallM => 11,
            Exception::InstructionPageFault => 12,
            Exception::LoadPageFault => 13,
            Exception::StorePageFault => 15,
        }
    }
    pub fn from_u32(x: u32) -> Option<Exception> {
        match x {
            0 => Some(Exception::InstructionAddressMisaligned),
            1 => Some(Exception::InstructionAccessFault),
            2 => Some(Exception::IllegalInstruction),
            3 => Some(Exception::Breakpoint),
            4 => Some(Exception::LoadAccessMisaligned),
            5 => Some(Exception::LoadAccessFault),
            6 => Some(Exception::StoreAddressMisalinged),
            7 => Some(Exception::StoreAccessFault),
            8 => Some(Exception::EnvironmentCallU),
            9 => Some(Exception::EnvironmentCallS),
            11 => Some(Exception::EnvironmentCallM),
            12 => Some(Exception::InstructionPageFault),
            13 => Some(Exception::LoadPageFault),
            15 => Some(Exception::StorePageFault),
            _ => None,
        }
    }
}
