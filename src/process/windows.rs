//! Owned Windows job. The launcher is suspended until assignment succeeds, so
//! it cannot spawn descendants outside the job during startup.
use anyhow::{Context, Result, bail};
use std::{
    ffi::c_void,
    mem::size_of,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    ptr,
};

type Handle = *mut c_void;
#[repr(C)]
#[derive(Default)]
struct BasicLimits {
    process_time: i64,
    job_time: i64,
    flags: u32,
    min_working_set: usize,
    max_working_set: usize,
    active_process_limit: u32,
    affinity: usize,
    priority: u32,
    scheduling: u32,
}
#[repr(C)]
#[derive(Default)]
struct ExtendedLimits {
    basic: BasicLimits,
    io: [u64; 6],
    process_memory: usize,
    job_memory: usize,
    peak_process_memory: usize,
    peak_job_memory: usize,
}
#[repr(C)]
#[derive(Default)]
struct ThreadEntry {
    size: u32,
    usage: u32,
    id: u32,
    process: u32,
    base_priority: i32,
    delta_priority: i32,
    flags: u32,
}
#[repr(C)]
#[derive(Default)]
struct Accounting {
    times: [i64; 4],
    page_faults: u32,
    total_processes: u32,
    active_processes: u32,
    terminated_processes: u32,
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateJobObjectW(attributes: *const c_void, name: *const u16) -> Handle;
    fn SetInformationJobObject(job: Handle, class: i32, info: *const c_void, size: u32) -> i32;
    fn AssignProcessToJobObject(job: Handle, process: Handle) -> i32;
    fn QueryInformationJobObject(
        job: Handle,
        class: i32,
        info: *mut c_void,
        size: u32,
        returned: *mut u32,
    ) -> i32;
    fn TerminateJobObject(job: Handle, exit_code: u32) -> i32;
    fn CreateToolhelp32Snapshot(flags: u32, process: u32) -> Handle;
    fn Thread32First(snapshot: Handle, entry: *mut ThreadEntry) -> i32;
    fn Thread32Next(snapshot: Handle, entry: *mut ThreadEntry) -> i32;
    fn OpenThread(access: u32, inherit: i32, id: u32) -> Handle;
    fn ResumeThread(thread: Handle) -> u32;
}

pub(super) struct Job(OwnedHandle);
impl Job {
    pub(super) fn new() -> Result<Self> {
        // SAFETY: no name/security attributes; check the returned owned handle.
        let raw = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };
        if raw.is_null() {
            return Err(std::io::Error::last_os_error()).context("create tool job");
        }
        let job = Self(unsafe { OwnedHandle::from_raw_handle(raw) });
        let limits = ExtendedLimits {
            basic: BasicLimits {
                flags: 0x2000,
                ..Default::default()
            },
            ..Default::default()
        };
        // JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE. No breakaway is allowed.
        if unsafe {
            SetInformationJobObject(
                job.0.as_raw_handle(),
                9,
                (&limits as *const ExtendedLimits).cast(),
                size_of::<ExtendedLimits>() as u32,
            )
        } == 0
        {
            return Err(std::io::Error::last_os_error()).context("configure tool job");
        }
        Ok(job)
    }
    pub(super) fn attach_and_resume(&self, child: &tokio::process::Child) -> Result<()> {
        let handle = child
            .raw_handle()
            .context("tool process handle unavailable")?;
        let pid = child.id().context("tool process id unavailable")?;
        // SAFETY: handles are live; the launcher was created suspended.
        if unsafe { AssignProcessToJobObject(self.0.as_raw_handle(), handle) } == 0 {
            return Err(std::io::Error::last_os_error())
                .context("assign suspended tool to owned job");
        }
        let raw = unsafe { CreateToolhelp32Snapshot(4, 0) }; // TH32CS_SNAPTHREAD
        if raw == -1isize as Handle {
            return Err(std::io::Error::last_os_error()).context("inspect suspended tool thread");
        }
        let snapshot = unsafe { OwnedHandle::from_raw_handle(raw) };
        let mut entry = ThreadEntry {
            size: size_of::<ThreadEntry>() as u32,
            ..Default::default()
        };
        let mut next = unsafe { Thread32First(snapshot.as_raw_handle(), &mut entry) };
        let mut ids = Vec::new();
        while next != 0 {
            if entry.process == pid {
                ids.push(entry.id);
            }
            next = unsafe { Thread32Next(snapshot.as_raw_handle(), &mut entry) };
        }
        if ids.len() != 1 {
            bail!("suspended tool must have exactly one initial thread");
        }
        let raw = unsafe { OpenThread(2, 0, ids[0]) }; // THREAD_SUSPEND_RESUME
        if raw.is_null() {
            return Err(std::io::Error::last_os_error()).context("open suspended tool thread");
        }
        let thread = unsafe { OwnedHandle::from_raw_handle(raw) };
        if unsafe { ResumeThread(thread.as_raw_handle()) } != 1 {
            bail!("could not resume the suspended tool thread exactly once");
        }
        Ok(())
    }
    pub(super) fn active(&self) -> Result<bool> {
        let mut accounting = Accounting::default();
        if unsafe {
            QueryInformationJobObject(
                self.0.as_raw_handle(),
                1,
                (&mut accounting as *mut Accounting).cast(),
                size_of::<Accounting>() as u32,
                ptr::null_mut(),
            )
        } == 0
        {
            return Err(std::io::Error::last_os_error()).context("inspect owned tool job cleanup");
        }
        Ok(accounting.active_processes != 0)
    }
    pub(super) fn terminate(&self) -> Result<()> {
        if unsafe { TerminateJobObject(self.0.as_raw_handle(), 1) } == 0 {
            return Err(std::io::Error::last_os_error()).context("terminate owned tool job");
        }
        Ok(())
    }
}
