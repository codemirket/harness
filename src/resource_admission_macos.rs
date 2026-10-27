//! Local libproc observations, never process-control authority.
use super::{Error, Identity, MAX_PROCESSES, NativeSample, Pressure, SampleError, Sampler};

pub(super) struct Native;
// Apple SDK sys/proc_info.h: PROC_PPID_ONLY is libproc selector 6.
const PROC_PPID_ONLY: u32 = 6;

fn bsd(pid: i32) -> Result<libc::proc_bsdinfo, SampleError> {
    let mut info = std::mem::MaybeUninit::<libc::proc_bsdinfo>::zeroed();
    let size = std::mem::size_of::<libc::proc_bsdinfo>();
    // libproc writes this fixed C structure only on an exact-size success.
    let read = unsafe {
        *libc::__error() = 0;
        libc::proc_pidinfo(
            pid,
            libc::PROC_PIDTBSDINFO,
            0,
            info.as_mut_ptr().cast(),
            size as i32,
        )
    };
    if read != size as i32 {
        if unsafe { *libc::__error() } == libc::ESRCH {
            return Err(SampleError::Gone);
        }
        return Err(
            Error::Unknown(format!("native process identity unavailable for PID {pid}")).into(),
        );
    }
    let info = unsafe { info.assume_init() };
    if info.pbi_pid != pid as u32 || info.pbi_uid != unsafe { libc::geteuid() } {
        return Err(Error::Unknown("native process identity or owner differs".into()).into());
    }
    Ok(info)
}

fn identity(info: &libc::proc_bsdinfo) -> Identity {
    Identity {
        pid: info.pbi_pid as i32,
        seconds: info.pbi_start_tvsec,
        micros: info.pbi_start_tvusec,
    }
}

pub(super) fn current() -> Result<Identity, Error> {
    Ok(identity(
        &bsd(std::process::id() as i32).map_err(SampleError::into_error)?,
    ))
}

impl Sampler for Native {
    fn sample(&self, pid: i32) -> Result<NativeSample, SampleError> {
        let before = bsd(pid)?;
        let mut task = std::mem::MaybeUninit::<libc::proc_taskinfo>::zeroed();
        let size = std::mem::size_of::<libc::proc_taskinfo>();
        let read = unsafe {
            *libc::__error() = 0;
            libc::proc_pidinfo(
                pid,
                libc::PROC_PIDTASKINFO,
                0,
                task.as_mut_ptr().cast(),
                size as i32,
            )
        };
        if read != size as i32 {
            if unsafe { *libc::__error() } == libc::ESRCH {
                return Err(SampleError::Gone);
            }
            return Err(Error::Unknown(format!(
                "native resident memory unavailable for PID {pid}"
            ))
            .into());
        }
        let after = bsd(pid)?;
        if identity(&before) != identity(&after) || before.pbi_ppid != after.pbi_ppid {
            return Err(SampleError::Changed);
        }
        Ok(NativeSample {
            identity: identity(&after),
            parent: after.pbi_ppid as i32,
            bytes: unsafe { task.assume_init() }.pti_resident_size,
        })
    }

    fn children(&self, pid: i32) -> Result<Vec<i32>, SampleError> {
        // proc_listpids returns bytes, unlike the convenience listchildpids
        // wrapper's count. A full buffer is ambiguous and therefore refused.
        let mut pids = vec![0i32; MAX_PROCESSES + 1];
        let capacity = std::mem::size_of_val(pids.as_slice());
        let read = unsafe {
            *libc::__error() = 0;
            libc::proc_listpids(
                PROC_PPID_ONLY,
                pid as u32,
                pids.as_mut_ptr().cast(),
                capacity as i32,
            )
        };
        if read == 0 && unsafe { *libc::__error() } == libc::ESRCH {
            return Err(SampleError::Gone);
        }
        if read < 0
            || (read == 0 && unsafe { *libc::__error() } != 0)
            || read as usize >= capacity
            || !(read as usize).is_multiple_of(4)
        {
            return Err(Error::Unknown(
                "native child inventory unavailable or exceeds bound".into(),
            )
            .into());
        }
        pids.truncate(read as usize / 4);
        if pids.iter().any(|pid| *pid <= 0) {
            return Err(
                Error::Unknown("native child inventory contains invalid identity".into()).into(),
            );
        }
        pids.sort_unstable();
        pids.dedup();
        Ok(pids)
    }
}

pub(super) fn pressure() -> Result<Pressure, Error> {
    let mut level = 0i32;
    let mut size = std::mem::size_of::<i32>();
    let code = unsafe {
        libc::sysctlbyname(
            c"kern.memorystatus_vm_pressure_level".as_ptr(),
            (&mut level as *mut i32).cast(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if code != 0 || size != std::mem::size_of::<i32>() {
        return Err(Error::Unknown("macOS memory pressure unavailable".into()));
    }
    match level {
        1 => Ok(Pressure::Normal),
        2 => Ok(Pressure::Warning),
        4 => Ok(Pressure::Critical),
        _ => Err(Error::Unknown("macOS memory pressure unknown".into())),
    }
}
