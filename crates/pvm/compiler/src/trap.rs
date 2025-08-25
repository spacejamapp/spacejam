//! Minimal trap handling using setjmp/longjmp and SIGSEGV following Wasmtime's approach

#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]

use std::cell::Cell;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

include!(concat!(env!("OUT_DIR"), "/setjmp.rs"));

/// Information about a trap that occurred
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrapInfo {
    /// The signal that was caught
    pub signal: i32,
    /// The fault address (if available)
    pub address: *mut libc::c_void,
    /// Additional signal info code
    pub code: i32,
}

thread_local! {
    /// Thread-local atomic pointer to jmp_buf
    static JMP_BUF_PTR: AtomicPtr<libc::c_void> = AtomicPtr::new(ptr::null_mut());
    /// Thread-local trap info storage
    static TRAP_INFO: Cell<Option<TrapInfo>> = Cell::new(None);
    /// Thread-local result storage for passing results back from C
    static RESULT_STORAGE: Cell<*mut libc::c_void> = Cell::new(ptr::null_mut());
}

/// Execute a function with SIGSEGV trap protection
pub fn with<F, T>(f: F) -> Result<T, TrapInfo>
where
    F: FnOnce() -> T,
{
    // Install signal handler once
    static HANDLER_INSTALLED: std::sync::Once = std::sync::Once::new();
    HANDLER_INSTALLED.call_once(|| unsafe {
        if pvm_install_sigsegv_handler(Some(sigsegv_handler)) != 0 {
            panic!("Failed to install SIGSEGV handler");
        }
    });

    // Clear any previous trap info and result
    TRAP_INFO.with(|info| info.set(None));
    RESULT_STORAGE.with(|storage| storage.set(ptr::null_mut()));

    // Create storage for the result
    let mut result_storage: Option<T> = None;
    let result_ptr = &mut result_storage as *mut Option<T> as *mut libc::c_void;
    RESULT_STORAGE.with(|storage| storage.set(result_ptr));
    let success = unsafe {
        let mut jmp_buf_storage: *mut libc::c_void = ptr::null_mut();
        let boxed_f = Box::new(f);
        let f_ptr = Box::into_raw(boxed_f) as *mut libc::c_void;
        let result = pvm_setjmp(
            &mut jmp_buf_storage as *mut *mut libc::c_void,
            Some(execute_closure::<F, T>),
            f_ptr,
            ptr::null_mut(),
        );

        result
    };

    // Clear jmp_buf and result storage
    JMP_BUF_PTR.with(|ptr| {
        ptr.store(ptr::null_mut(), Ordering::SeqCst);
    });
    RESULT_STORAGE.with(|storage| storage.set(ptr::null_mut()));
    if success {
        match result_storage {
            Some(value) => Ok(value),
            None => Err(TrapInfo {
                signal: -1,
                address: ptr::null_mut(),
                code: -1,
            }),
        }
    } else {
        TRAP_INFO.with(|info| {
            Err(info.get().unwrap_or(TrapInfo {
                signal: libc::SIGSEGV,
                address: ptr::null_mut(),
                code: 0,
            }))
        })
    }
}

/// Execute the Rust closure within the setjmp context
unsafe extern "C" fn execute_closure<F, T>(
    payload: *mut libc::c_void,
    jmp_buf: *mut libc::c_void,
) -> bool
where
    F: FnOnce() -> T,
{
    JMP_BUF_PTR.with(|ptr| {
        ptr.store(jmp_buf, Ordering::SeqCst);
    });

    // Take the ownership of the closure
    let f_ptr = payload as *mut F;
    let boxed_f = Box::from_raw(f_ptr);
    let f = *boxed_f;

    // Execute the closure
    let result = f();
    RESULT_STORAGE.with(|storage| {
        let result_ptr = storage.get() as *mut Option<T>;
        if !result_ptr.is_null() {
            *result_ptr = Some(result);
        }
    });

    true
}

/// SIGSEGV signal handler
extern "C" fn sigsegv_handler(sig: libc::c_int, info: *mut siginfo_t, _context: *mut libc::c_void) {
    let fault_addr = unsafe { (*info)._sifields._sigfault.si_addr };
    let fault_code = unsafe { (*info).si_code };
    TRAP_INFO.with(|trap_info| {
        trap_info.set(Some(TrapInfo {
            signal: sig,
            address: fault_addr,
            code: fault_code,
        }));
    });

    // get jmp_buf and jump back
    JMP_BUF_PTR.with(|ptr| {
        let jmp_buf_ptr = ptr.load(Ordering::SeqCst);
        if !jmp_buf_ptr.is_null() {
            unsafe {
                pvm_longjmp(jmp_buf_ptr);
            }
        }
    });

    // If we get here, re-raise the signal
    unsafe {
        libc::raise(sig);
    }
}
