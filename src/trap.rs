use stvec;

extern "C" {
    static trap_entry: u8;
}

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

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Register {
    pub int_regs: [u32; 32],
    pub float_regs: [u32; 32],
}

impl Register {
    pub fn zeros() -> Register {
        Register {
            int_regs: [0; 32],
            float_regs: [0; 32],
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TrapFrame {
    pub pc: u32,
    pub sp: u32,
    pub regs: Register,
}

impl TrapFrame {
    pub fn new(pc: u32, sp: u32) -> TrapFrame {
        TrapFrame {
            pc,
            sp,
            regs: Register::zeros(),
        }
    }
}

pub fn trap_init() {
    println!("setting stvec");
    stvec::STVEC::set_mode(stvec::Mode::Direct);
    let trap_entry_addr = unsafe { (&trap_entry as *const u8) } as u32;
    println!("trap entry: {:x}", trap_entry_addr);
    stvec::STVEC::set_trap_base(trap_entry_addr);
}

pub fn trap(tf: TrapFrame) -> ! {
    println!("entering trap");
    println!("{:?}", &tf);

    panic!("failed to run process");
}

// 最初にepcをバックアップして、例外を無効にしてから処理
global_asm!(
    r#"
.global trap_entry
trap_entry:
    csrrw x0, sscratch, sp
    lui     sp, %hi(interrupt_stack_end)
    addi    sp, sp, %lo(interrupt_stack_end)
    addi sp, sp, -128
    sw x0, 0(sp)
    sw x1, 4(sp)
    
    csrrs x1, scause, x0 
    sw x1, 8(sp)

    sw x3, 12(sp)
    sw x4, 16(sp)
    sw x5, 20(sp)
    sw x6, 24(sp)
    sw x7, 28(sp)
    sw x8, 32(sp)
    sw x9, 36(sp)
    sw x10, 40(sp)
    sw x11, 44(sp)
    sw x12, 48(sp)
    sw x13, 52(sp)
    sw x14, 56(sp)
    sw x15, 60(sp)
    sw x16, 64(sp)
    sw x17, 68(sp)
    sw x18, 72(sp)
    sw x19, 76(sp)
    sw x20, 80(sp)
    sw x21, 84(sp)
    sw x22, 88(sp)
    sw x23, 92(sp)
    sw x24, 96(sp)
    sw x25, 100(sp)
    sw x26, 104(sp)
    sw x27, 108(sp)
    sw x28, 112(sp)
    sw x29, 116(sp)
    sw x30, 120(sp)
    sw x31, 124(sp)

    mv a0, sp
    call trap_entry_rust
"#
);

#[no_mangle]
extern "C" fn trap_entry_rust(regs: *const Register) -> ! {
    // TODO: prohibit interrupt
    let sp;
    let pc;
    unsafe {
        asm!(
            "
            csrrs $0, sscratch, x0\n
            csrrs $1, sepc, x0\n
            "
            : "=&r"(sp), "=&r"(pc)
        );
    }
    let mut tf = TrapFrame::new(pc, sp);
    tf.regs = unsafe { *regs };
    trap(tf)
}

pub unsafe fn pop_trap_frame(tf: &TrapFrame) -> ! {
    // TODO: prohibit interrupt

    asm!(
        "
        csrrw x0, sscratch, $0\n
        csrrw x0, sepc, $1\n
        "
        :
        : "r"(tf.sp), "r"(tf.pc)

    );

    asm!(
        "
        mv sp, $0\n

        lw x0, 0(sp)\n
        lw x1, 4(sp)\n
        lw x3, 12(sp)\n
        lw x4, 16(sp)\n
        lw x5, 20(sp)\n
        lw x6, 24(sp)\n
        lw x7, 28(sp)\n
        lw x8, 32(sp)\n
        lw x9, 36(sp)\n
        lw x10, 40(sp)\n
        lw x11, 44(sp)\n
        lw x12, 48(sp)\n
        lw x13, 52(sp)\n
        lw x14, 56(sp)\n
        lw x15, 60(sp)\n
        lw x16, 64(sp)\n
        lw x17, 68(sp)\n
        lw x18, 72(sp)\n
        lw x19, 76(sp)\n
        lw x20, 80(sp)\n
        lw x21, 84(sp)\n
        lw x22, 88(sp)\n
        lw x23, 92(sp)\n
        lw x24, 96(sp)\n
        lw x25, 100(sp)\n
        lw x26, 104(sp)\n
        lw x27, 108(sp)\n
        lw x28, 112(sp)\n
        lw x29, 116(sp)\n
        lw x30, 120(sp)\n
        lw x31, 124(sp)\n

        addi sp, sp, 128\n

        csrrs sp, sscratch, x0\n
        sret
    "
    :
    : "r"(&tf.regs as *const Register as usize)
    );

    /*
        flw f0, 0(sp)\n
        flw f1, 4(sp)\n
        flw f3, 12(sp)\n
        flw f4, 16(sp)\n
        flw f5, 20(sp)\n
        flw f6, 24(sp)\n
        flw f7, 28(sp)\n
        flw f8, 32(sp)\n
        flw f9, 36(sp)\n
        flw f10, 40(sp)\n
        flw f11, 44(sp)\n
        flw f12, 48(sp)\n
        flw f13, 52(sp)\n
        flw f14, 56(sp)\n
        flw f15, 60(sp)\n
        flw f16, 64(sp)\n
        flw f17, 68(sp)\n
        flw f18, 72(sp)\n
        flw f19, 76(sp)\n
        flw f20, 80(sp)\n
        flw f21, 84(sp)\n
        flw f22, 88(sp)\n
        flw f23, 92(sp)\n
        flw f24, 96(sp)\n
        flw f25, 100(sp)\n
        flw f26, 104(sp)\n
        flw f27, 108(sp)\n
        flw f28, 112(sp)\n
        flw f29, 116(sp)\n
        flw f30, 120(sp)\n
        flw f31, 124(sp)\n
    */

    panic!("failed to sret")
}
