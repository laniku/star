use crate::scheduler::{TaskState, TASKS, CURRENT_TASK};
use crate::ramfs::ramfs_mut;

pub const SYS_READ: usize = 0;
pub const SYS_WRITE: usize = 64;
pub const SYS_OPEN: usize = 2;
pub const SYS_CLOSE: usize = 3;
pub const SYS_EXIT: usize = 93;
pub const SYS_FORK: usize = 220;
pub const SYS_EXECVE: usize = 221;
pub const SYS_WAIT: usize = 260;
pub const SYS_GETPID: usize = 172;
pub const SYS_BRK: usize = 9;
pub const SYS_RAMFS_CREATE: usize = 1000;
pub const SYS_RAMFS_READ: usize = 1001;
pub const SYS_RAMFS_WRITE: usize = 1002;
pub const SYS_RAMFS_LIST: usize = 1003;

pub fn handle_syscall(syscall_num: usize, arg1: usize, arg2: usize, arg3: usize) -> usize {
    match syscall_num {
        SYS_READ => sys_read(arg1, arg2, arg3),
        SYS_WRITE => sys_write(arg1, arg2, arg3),
        SYS_EXIT => sys_exit(arg1, arg2, arg3),
        SYS_GETPID => sys_getpid(arg1, arg2, arg3),
        SYS_FORK => sys_fork(arg1, arg2, arg3),
        SYS_EXECVE => sys_execve(arg1, arg2, arg3),
        SYS_WAIT => sys_wait(arg1, arg2, arg3),
        SYS_RAMFS_CREATE => sys_ramfs_create(arg1, arg2, arg3),
        SYS_RAMFS_READ => sys_ramfs_read(arg1, arg2, arg3),
        SYS_RAMFS_WRITE => sys_ramfs_write(arg1, arg2, arg3),
        SYS_RAMFS_LIST => sys_ramfs_list(arg1, arg2, arg3),
        _ => usize::MAX,
    }
}

fn sys_read(fd: usize, buf: usize, len: usize) -> usize {
    if fd == 0 {
        let mut bytes_read = 0;
        let buffer = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, len) };
        
        for i in 0..len {
            let ch = crate::user_loader::getchar();
            buffer[i] = ch;
            bytes_read += 1;
            
            if ch == b'\n' || ch == b'\r' {
                break;
            }
        }
        
        bytes_read
    } else {
        0
    }
}

fn sys_write(fd: usize, buf: usize, len: usize) -> usize {
    if fd == 1 {
        let slice = unsafe { core::slice::from_raw_parts(buf as *const u8, len) };
        if let Ok(s) = core::str::from_utf8(slice) {
            crate::println!("{}", s);
            len
        } else {
            0
        }
    } else {
        0
    }
}

fn sys_exit(_status: usize, _arg2: usize, _arg3: usize) -> usize {
    unsafe {
        let cur = CURRENT_TASK;
        let task = &mut TASKS[cur];
        task.state = TaskState::Exited;
        task.active = false;
        let next = crate::scheduler::next_task();
        crate::scheduler::switch_to_task(next);
    }
}

fn sys_getpid(_arg1: usize, _arg2: usize, _arg3: usize) -> usize {
    unsafe { CURRENT_TASK }
}

fn sys_fork(_arg1: usize, _arg2: usize, _arg3: usize) -> usize {
    unsafe {
        let parent = CURRENT_TASK;
        for (i, task) in TASKS.iter_mut().enumerate() {
            if !task.active {
                *task = TASKS[parent].clone();
                task.pid = i;
                task.ppid = parent;
                task.state = TaskState::Ready;
                task.active = true;
                return i;
            }
        }
    }
    usize::MAX
}

fn sys_execve(entry: usize, _argv: usize, _envp: usize) -> usize {
    unsafe {
        let cur = CURRENT_TASK;
        let task = &mut TASKS[cur];
        task.ctx.pc = entry;
        task.state = TaskState::Ready;
        0
    }
}

fn sys_wait(pid: usize, _arg2: usize, _arg3: usize) -> usize {
    unsafe {
        for task in TASKS.iter() {
            if task.pid == pid && task.state != TaskState::Exited {
                return 0;
            }
        }
        1
    }
}

fn sys_ramfs_create(name_ptr: usize, data_ptr: usize, data_len: usize) -> usize {
    let name = unsafe {
        let bytes = core::slice::from_raw_parts(name_ptr as *const u8, 32);
        match core::str::from_utf8(bytes) {
            Ok(s) => s.trim_end_matches(char::from(0)),
            Err(_) => "",
        }
    };
    let data = unsafe { core::slice::from_raw_parts(data_ptr as *const u8, data_len) };
    let fs = ramfs_mut();
    fs.create_file(name, data);
    0
}

fn sys_ramfs_read(name_ptr: usize, buf_ptr: usize, buf_len: usize) -> usize {
    let name = unsafe {
        let bytes = core::slice::from_raw_parts(name_ptr as *const u8, 32);
        match core::str::from_utf8(bytes) {
            Ok(s) => s.trim_end_matches(char::from(0)),
            Err(_) => "",
        }
    };
    let fs = ramfs_mut();
    if let Some(data) = fs.read_file(name) {
        let copy_len = core::cmp::min(buf_len, data.len());
        unsafe {
            core::ptr::copy_nonoverlapping(data.as_ptr(), buf_ptr as *mut u8, copy_len);
        }
        copy_len
    } else {
        0
    }
}

fn sys_ramfs_write(name_ptr: usize, data_ptr: usize, data_len: usize) -> usize {
    let name = unsafe {
        let bytes = core::slice::from_raw_parts(name_ptr as *const u8, 32);
        match core::str::from_utf8(bytes) {
            Ok(s) => s.trim_end_matches(char::from(0)),
            Err(_) => "",
        }
    };
    let data = unsafe { core::slice::from_raw_parts(data_ptr as *const u8, data_len) };
    let fs = ramfs_mut();
    if fs.write_file(name, data) {
        0
    } else {
        usize::MAX
    }
}

fn sys_ramfs_list(buf_ptr: usize, buf_len: usize, _unused: usize) -> usize {
    let fs = ramfs_mut();
    let files = fs.list_files();
    let mut total = 0;
    for name in files {
        let bytes = name.as_bytes();
        let copy_len = core::cmp::min(buf_len - total, bytes.len());
        if copy_len == 0 { break; }
        unsafe {
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), (buf_ptr + total) as *mut u8, copy_len);
        }
        total += copy_len;
        if total < buf_len {
            unsafe { *(buf_ptr as *mut u8).add(total) = b'\n'; }
            total += 1;
        }
    }
    total
}