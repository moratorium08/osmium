use paging;

pub const USER_STACK_TOP: u32 = 0xe0000000;
pub const USER_STACK_SIZE: u32 = paging::PGSIZE as u32 * 16;
pub const USER_SATCK_BOTTOMN: u32 = USER_STACK_TOP + USER_STACK_SIZE - 4;
