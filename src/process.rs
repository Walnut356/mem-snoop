use std::{
    ffi::c_void,
    io,
    mem::{size_of, MaybeUninit, transmute_copy},
    ptr::NonNull, error,
};
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE, HMODULE},
    System::{
        ProcessStatus::{EnumProcessModules, EnumProcesses, GetModuleBaseNameA},
        Threading::{OpenProcess, PROCESS_ACCESS_RIGHTS, PROCESS_VM_READ, PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameA, PROCESS_NAME_FORMAT},
    },
};

type DWORD = u32;
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

pub fn enum_processes() -> io::Result<Vec<u32>> {
    let mut pids = Vec::<DWORD>::with_capacity(1024);
    let mut size = 0;
    if unsafe {
        EnumProcesses(
            pids.as_mut_ptr(),
            (pids.capacity() * size_of::<DWORD>()) as u32,
            &mut size,
        )
    }
    .is_err()
    {
        return Err(io::Error::last_os_error());
    }

    let count = size as usize / size_of::<DWORD>();

    unsafe { pids.set_len(count) };

    Ok(pids)
}

pub struct Process {
    pub pid: u32,
    pub handle: HANDLE,
}

impl Process {
    pub fn open(pid: u32) -> Result<Self> {
        Ok(unsafe { OpenProcess(PROCESS_VM_READ | PROCESS_QUERY_INFORMATION, false, pid) }
            .map(|handle| Self { pid, handle })?)
    }

    pub fn name(&self) -> Result<String> {
        let mut module = MaybeUninit::<HMODULE>::uninit();
        let mut size = 0;

        unsafe {
            EnumProcessModules(
                self.handle,
                module.as_mut_ptr(),
                size_of::<HMODULE>() as u32,
                &mut size,
            )?;
        }


        let module = unsafe { module.assume_init() };

        // the window-rs api call uses buffer's length automatically (rather than its capacity)
        // so we have to initialize it with something
        let mut buffer = vec![0; 64];

        let length = unsafe { GetModuleBaseNameA(self.handle, module, &mut buffer) };

        if length == 0 {
            return Err(Box::new(io::Error::last_os_error()));
        }

        unsafe { buffer.set_len(length as usize) };
        Ok(String::from_utf8(buffer).unwrap())
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.handle).unwrap() };
    }
}
