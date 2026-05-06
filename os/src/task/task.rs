//! Types related to task management

use crate::config::MAX_SYSCALL_ID;
use super::TaskContext;

/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// syscall_id -> called_count
    pub syscall_count: [usize; MAX_SYSCALL_ID],
}

impl TaskControlBlock {
    /// 更新当前任务系统调用统计数
    pub fn increase_syscall_count(&mut self, syscall_id: usize) {
        if syscall_id >= MAX_SYSCALL_ID {
            return;
        }
        self.syscall_count[syscall_id] += 1;
    }

    /// 获取当前任务系统调用统计数
    pub fn get_syscall_count(&self, syscall_id: usize) -> usize {
        if syscall_id >= MAX_SYSCALL_ID {
            0
        } else {
            self.syscall_count[syscall_id]
        }
    }
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}
