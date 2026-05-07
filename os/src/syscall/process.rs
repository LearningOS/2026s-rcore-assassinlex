//! Process management syscalls

use core::mem::size_of;
use crate::config::PAGE_SIZE;
use crate::mm::{translated_byte_buffer, PTEFlags, PageTable, VirtAddr};
use crate::task::{change_program_brk, current_user_token, exit_current_and_run_next, get_syscall_count, mmap, munmap, suspend_current_and_run_next};
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let time_val = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let src = unsafe {
        core::slice::from_raw_parts(
            &time_val as *const TimeVal as *const u8,
            size_of::<TimeVal>()
        )
    };
    let token = current_user_token();
    let buffers = translated_byte_buffer(token, ts as *const u8, size_of::<TimeVal>());
    let mut start = 0usize;
    for buffer in buffers.into_iter() {
        let len = buffer.len();
        buffer.copy_from_slice(&src[start..start+len]);
        start += len;
        if start >= src.len() {
            break;
        }
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    let va = VirtAddr::from(id);
    let vpn = va.floor();
    let offset = va.page_offset();
    let page_table = PageTable::from_token(current_user_token());
    let pte = page_table.translate(vpn).unwrap();
    let exit_code = match trace_request {
        // 读取一个字节
        0 => {
            if !pte.flags().contains(PTEFlags::V | PTEFlags::U | PTEFlags::R) {
                return -1;
            }
            pte.ppn().get_bytes_array()[offset] as isize
        },
        // 写入 data 最低位的一个字节
        1 => {
            if !pte.flags().contains(PTEFlags::V | PTEFlags::U | PTEFlags::W) {
                return -1;
            }
            pte.ppn().get_bytes_array()[offset] = data as u8;
            0
        },
        // 统计当前任务 syscall_id 调用次数，sys_trace 也要计入
        2 => get_syscall_count(id) as isize,
        _ => -1
    };
    exit_code
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    if port & !0x7 != 0 || port & 0x7 == 0 {
        return -1;
    }
    mmap(start, start+len, port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    munmap(start, start+len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
