use core::fmt;
use csr;
use csr::{CSRRead, CSRWrite};
use kernel;
use stvec;
use syscall;

extern "C" {
    static trap_entry: u8;
}

#[derive(Copy, Clone, Debug)]
pub enum Trap {
    Exception(Exception),
    Interruption(Interruption),
}

impl fmt::Display for Trap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl Trap {
    pub fn from_u32(x: u32) -> Option<Trap> {
        if (x >> 31) == 1 {
            let int = Interruption::from_u32(x & !(1 << 31));
            match int {
                Some(int) => Some(Trap::Interruption(int)),
                None => None,
            }
        } else {
            let exc = Exception::from_u32(x & !(1 << 31));
            match exc {
                Some(int) => Some(Trap::Exception(int)),
                None => None,
            }
        }
    }
    pub fn to_str(self) -> &'static str {
        match self {
            Trap::Exception(e) => e.to_str(),
            Trap::Interruption(i) => i.to_str(),
        }
    }
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

impl fmt::Display for Interruption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl Interruption {
    pub fn to_str(self) -> &'static str {
        match self {
            Interruption::UserSoftware => "User Software Interruption",
            Interruption::SupervisorSoftware => "Supervisor Software Interruption",
            Interruption::MachineSoftware => "Machine Software Interruption",
            Interruption::UserTimer => "User Timer Interruption",
            Interruption::SupervisorTimer => "Supervisor Timer Interruption",
            Interruption::MachineTimer => "Machine Timer Interruption",
            Interruption::UserExternal => "User External",
            Interruption::SupervisorExternal => "Supvervisor External",
            Interruption::MachineExternal => "Machine External",
        }
    }
    pub fn to_u32(self) -> u32 {
        match self {
            Interruption::UserSoftware => 1 << 0,
            Interruption::SupervisorSoftware => 1 << 1,
            Interruption::MachineSoftware => 1 << 3,
            Interruption::UserTimer => 1 << 4,
            Interruption::SupervisorTimer => 1 << 5,
            Interruption::MachineTimer => 1 << 7,
            Interruption::UserExternal => 1 << 8,
            Interruption::SupervisorExternal => 1 << 9,
            Interruption::MachineExternal => 1 << 11,
        }
    }
    pub fn from_u32(x: u32) -> Option<Interruption> {
        match x {
            0b1 => Some(Interruption::UserSoftware),
            0b10 => Some(Interruption::SupervisorSoftware),
            0b1000 => Some(Interruption::MachineSoftware),
            0b10000 => Some(Interruption::UserTimer),
            0b100000 => Some(Interruption::SupervisorTimer),
            0b10000000 => Some(Interruption::MachineTimer),
            0b100000000 => Some(Interruption::UserExternal),
            0b1000000000 => Some(Interruption::SupervisorExternal),
            0b100000000000 => Some(Interruption::MachineExternal),
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

impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl Exception {
    pub fn to_str(self) -> &'static str {
        match self {
            Exception::InstructionAddressMisaligned => "Instruction Address Misaligned Exception",
            Exception::InstructionAccessFault => "Instruction Access Fault Exception",
            Exception::IllegalInstruction => "Illegal Instruction Exception",
            Exception::Breakpoint => "Breakpoint Exception",
            Exception::LoadAccessMisaligned => "Load Access Misaligned Exception",
            Exception::LoadAccessFault => "Load Access Fault Exception",
            Exception::StoreAddressMisalinged => "Store Address Misaligned Exception",
            Exception::StoreAccessFault => "Store Access Fault Exception",
            Exception::EnvironmentCallU => "Environment Call from User Mode Exception",
            Exception::EnvironmentCallS => "Environment Call from Supervisor Mode Exception",
            Exception::EnvironmentCallM => "Environment Call from Machine Mode Exception",
            Exception::InstructionPageFault => "Instruction Page Fault Exception",
            Exception::LoadPageFault => "Load Page Fault Exception",
            Exception::StorePageFault => "Store Page Fault Exception",
        }
    }
    pub fn to_u32(self) -> u32 {
        match self {
            Exception::InstructionAddressMisaligned => 0 << 1,
            Exception::InstructionAccessFault => 1 << 1,
            Exception::IllegalInstruction => 2 << 1,
            Exception::Breakpoint => 3 << 1,
            Exception::LoadAccessMisaligned => 4 << 1,
            Exception::LoadAccessFault => 5 << 1,
            Exception::StoreAddressMisalinged => 6 << 1,
            Exception::StoreAccessFault => 7 << 1,
            Exception::EnvironmentCallU => 8 << 1,
            Exception::EnvironmentCallS => 9 << 1,
            Exception::EnvironmentCallM => 11 << 1,
            Exception::InstructionPageFault => 12 << 1,
            Exception::LoadPageFault => 13 << 1,
            Exception::StorePageFault => 15 << 1,
        }
    }
    // use bitflags
    pub fn from_u32(x: u32) -> Option<Exception> {
        match x {
            0b1 => Some(Exception::InstructionAddressMisaligned),
            0b10 => Some(Exception::InstructionAccessFault),
            0b100 => Some(Exception::IllegalInstruction),
            0b1000 => Some(Exception::Breakpoint),
            0b10000 => Some(Exception::LoadAccessMisaligned),
            0b100000 => Some(Exception::LoadAccessFault),
            0b1000000 => Some(Exception::StoreAddressMisalinged),
            0b10000000 => Some(Exception::StoreAccessFault),
            0b100000000 => Some(Exception::EnvironmentCallU),
            0b1000000000 => Some(Exception::EnvironmentCallS),
            0b100000000000 => Some(Exception::EnvironmentCallM),
            0b1000000000000 => Some(Exception::InstructionPageFault),
            0b10000000000000 => Some(Exception::LoadPageFault),
            0b1000000000000000 => Some(Exception::StorePageFault),
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
    pub fn a0(&self) -> u32 {
        self.int_regs[10]
    }
    pub fn a1(&self) -> u32 {
        self.int_regs[11]
    }
    pub fn a2(&self) -> u32 {
        self.int_regs[12]
    }
    pub fn a3(&self) -> u32 {
        self.int_regs[13]
    }
    pub fn a4(&self) -> u32 {
        self.int_regs[14]
    }
    pub fn a5(&self) -> u32 {
        self.int_regs[15]
    }
    pub fn a6(&self) -> u32 {
        self.int_regs[16]
    }
    pub fn set_syscall_result(&mut self, x: u32) {
        self.int_regs[10] = x;
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
    dprintln!("setting stvec");
    stvec::STVEC::set_mode(stvec::Mode::Direct);
    let trap_entry_addr = unsafe { (&trap_entry as *const u8) } as u32;
    dprintln!("trap entry: {:x}", trap_entry_addr);
    stvec::STVEC::set_trap_base(trap_entry_addr);
}

pub fn handle_envcall(mut tf: TrapFrame) -> ! {
    let kernel = unsafe { kernel::get_kernel() };
    let e = match syscall::Syscall::from_trap_frame(&tf) {
        Ok(syscall) => syscall::syscall_dispatch(syscall, kernel, &tf),
        Err(e) => {
            dprintln!("failed to run env call: {}", e);
            Err(e)
        }
    };
    match e {
        Ok(result) => {
            tf.regs.set_syscall_result(result);
            kernel.update_current_process_trap_frame(tf);

            kernel.run_into_user()
        }
        Err(e) => {
            // handle kill process
            panic!("failed to do syscall: {}", e)
        }
    }
}

fn handle_store_page_fault(mut tf: TrapFrame) -> ! {
    let k = unsafe { kernel::get_kernel() };

    let stval = csr::stval::STVAL::read();
    //k.current_process.unwrap().mapper.check()
    unimplemented!()
}
fn handle_load_page_fault(mut tf: TrapFrame) -> ! {
    unimplemented!()
}
fn handle_instr_page_fault(mut tf: TrapFrame) -> ! {
    unimplemented!()
}
fn handle_access_fault(mut tf: TrapFrame) -> ! {
    unimplemented!()
}

fn exception_handler(exc: Exception, tf: TrapFrame) -> ! {
    match exc {
        Exception::EnvironmentCallU => handle_envcall(tf),
        Exception::LoadPageFault => handle_load_page_fault(tf),
        Exception::StorePageFault => handle_store_page_fault(tf),
        Exception::InstructionPageFault => handle_instr_page_fault(tf),
        Exception::LoadAccessFault
        | Exception::StoreAccessFault
        | Exception::InstructionAccessFault => handle_access_fault(tf),
        _ => panic!("{} is not supported", exc.to_str()),
    }
}

fn handle_timer(mut tf: TrapFrame) -> ! {
    unimplemented!()
}

fn interruption_handler(itrpt: Interruption, tf: TrapFrame) -> ! {
    match itrpt {
        Interruption::MachineTimer | Interruption::SupervisorTimer | Interruption::UserTimer => {
            handle_timer(tf)
        }
        _ => panic!("{} is not supported", itrpt.to_str()),
    }
}

fn trap(tf: TrapFrame) -> ! {
    dprintln!("entering trap");
    dprintln!("{:?}", &tf);

    let trap = Trap::from_u32(tf.regs.int_regs[2]).expect("failed to parse trap cause");
    dprintln!("caught trap: {}", trap);
    let sstatus: u32;
    let sie: u32;
    // TODO: create CSR wrapper
    unsafe {
        asm!(
            "
        csrrs $0, sstatus, x0\n
        csrrs $1, sie, x0\n
    "
        : "=&r"(sstatus), "=&r"(sie)
            );
    }
    dprintln!(
        "sepc = {:x}, stval = {:x}\nsstatus = {:x}, sie = {:x}, sp = {:x}",
        tf.pc,
        csr::stval::STVAL::read_csr(),
        sstatus,
        sie,
        tf.sp
    );

    match trap {
        Trap::Exception(e) => exception_handler(e, tf),
        Trap::Interruption(i) => interruption_handler(i, tf),
    }
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
