use paging;
use proc;
use trap;

pub struct Kernel<'a> {
    pub mapper: paging::Map<'a>,
    pub allocator: paging::Allocator<'a>,
    pub process_manager: proc::ProcessManager<'a>,
    pub current_process: Option<&'a mut proc::Process<'a>>,
}

impl<'a> Kernel<'a> {
    pub fn size_of() -> usize {
        use core::intrinsics::size_of_val;
        let dummy = 100usize;
        let p: &Kernel = unsafe { &*(dummy as *const Kernel) };
        unsafe { size_of_val(p) }
    }

    pub fn sched(&mut self) -> ! {
        unimplemented!()
    }

    pub fn run_into_user(&mut self) -> ! {
        match self.current_process {
            Some(ref mut p) => p.run(),
            None => self.sched(),
        }
    }

    pub fn update_current_process_trap_frame(&mut self, tf: trap::TrapFrame) {
        match self.current_process {
            Some(ref mut p) => p.trap_frame = tf,
            None => (),
        }
    }
}

static mut KERNEL: u32 = 0;

pub fn set_kernel_ptr(ptr: *const Kernel) {
    unsafe {
        KERNEL = ptr as u32;
    }
}

pub unsafe fn set_kernel(k: Kernel<'static>) {
    assert!(KERNEL != 0); // This is like a C :cry:

    let ptr: *mut Kernel = KERNEL as *mut Kernel;
    *ptr = k;
}

pub unsafe fn get_kernel<'a>() -> &'a mut Kernel<'static> {
    assert!(KERNEL != 0); // This is like a C :cry:

    let ptr: *mut Kernel = KERNEL as *mut Kernel;
    &mut *ptr
}
