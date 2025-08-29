#include <setjmp.h>
#include <signal.h>
#include <stdbool.h>
#include <stdint.h>

// Platform-specific setjmp/longjmp selection following Wasmtime's approach
#if (defined(__GNUC__) && !defined(__clang__)) || defined(__x86_64__) || defined(__aarch64__)
    #define PVM_GCC 1
#endif

#ifdef __APPLE__
    typedef long long pvm_jmp_buf[37];
    extern int _setjmp(pvm_jmp_buf env);
    extern void longjmp(pvm_jmp_buf env, int val) __attribute__((noreturn));
    #define platform_setjmp(buf) _setjmp(buf)
    #define platform_longjmp(buf, val) longjmp(buf, val)
    typedef pvm_jmp_buf platform_jmp_buf;
#elif defined(_WIN32)
    #define platform_setjmp(buf) setjmp(buf)
    #define platform_longjmp(buf, arg) longjmp(buf, arg)
    typedef jmp_buf platform_jmp_buf;
#elif defined(PVM_GCC) || defined(__x86_64__) || defined(__aarch64__)
    #define platform_setjmp(buf) __builtin_setjmp(buf)
    #define platform_longjmp(buf, arg) __builtin_longjmp(buf, arg)
    typedef void *platform_jmp_buf[5];
#else
    #define platform_setjmp(buf) sigsetjmp(buf, 0)
    #define platform_longjmp(buf, arg) siglongjmp(buf, arg)
    typedef sigjmp_buf platform_jmp_buf;
#endif

// Platform-specific signal info structure
#ifdef __APPLE__
    typedef struct
    {
        int si_signo;
        int si_code;
        union
        {
            struct
            {
                void *si_addr;
            } _sigfault;
        } _sifields;
    } pvm_siginfo_t;
#else
    typedef siginfo_t pvm_siginfo_t;
#endif

// Signal constants - platform specific
#ifdef __APPLE__
    #define PVM_SIGSEGV 11
    #define PVM_SIGBUS 10 // macOS uses 10 for SIGBUS
    #define PVM_SIGTRAP 5
    #define PVM_SA_SIGINFO 0x40 // SA_SIGINFO on macOS
    #define PVM_SA_NODEFER 0x10 // SA_NODEFER on macOS
#else
    #define PVM_SIGSEGV 11
    #define PVM_SIGBUS 7 // Linux uses 7 for SIGBUS
    #define PVM_SIGTRAP 5
    #define PVM_SA_SIGINFO SA_SIGINFO
    #define PVM_SA_NODEFER SA_NODEFER
#endif

// PVM trap handling API
bool setjmp_rs(void **buf_storage, bool (*body)(void *, void *), void *payload, void *callee);
void longjmp_rs(void *jmp_buf_ptr);
// Multi-signal handler for macOS support, single SIGSEGV for Linux compatibility
#ifdef __APPLE__
    int install_signal_handlers(void (*handler)(int, pvm_siginfo_t *, void *));
#else
    int install_signal_handlers(void (*handler)(int, siginfo_t*, void*));
#endif