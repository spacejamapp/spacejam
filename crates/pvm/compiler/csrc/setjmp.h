#ifndef PVM_SETJMP_H
#define PVM_SETJMP_H

// Minimal includes to avoid cross-compilation issues
#include <stdbool.h>
#include <stdint.h>

// Forward declarations to avoid including system headers
struct sigaction;
union sigval;

// Platform-specific signal info structure (simplified from siginfo_t)
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

// Platform-specific jmp_buf sizes (conservative estimates)
#ifdef __APPLE__
#ifdef __aarch64__
typedef long long pvm_jmp_buf[37]; // macOS ARM64
#else
typedef long long pvm_jmp_buf[37]; // macOS x86_64 (conservative)
#endif
#elif defined(__linux__)
#ifdef __aarch64__
typedef long long pvm_jmp_buf[22]; // Linux ARM64
#else
typedef long long pvm_jmp_buf[8]; // Linux x86_64
#endif
#else
typedef long long pvm_jmp_buf[64]; // Conservative fallback
#endif

// External system functions (avoid including headers)
extern int sigaction(int sig, const struct sigaction *act, struct sigaction *oldact);
extern int sigemptyset(void *set);
extern int raise(int sig);

// Platform-specific setjmp/longjmp functions
#ifdef __APPLE__
extern int _setjmp(pvm_jmp_buf env);
extern void longjmp(pvm_jmp_buf env, int val) __attribute__((noreturn));
#define platform_setjmp(buf) _setjmp(buf)
#define platform_longjmp(buf, val) longjmp(buf, val)
#else
extern int sigsetjmp(pvm_jmp_buf env, int savesigs);
extern void siglongjmp(pvm_jmp_buf env, int val) __attribute__((noreturn));
#define platform_setjmp(buf) sigsetjmp(buf, 0)
#define platform_longjmp(buf, val) siglongjmp(buf, val)
#endif

typedef pvm_jmp_buf platform_jmp_buf;

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
#define PVM_SA_SIGINFO 4
#define PVM_SA_NODEFER 0x40000000
#endif

// PVM trap handling API
bool pvm_setjmp(void **buf_storage, bool (*body)(void *, void *), void *payload, void *callee);
void pvm_longjmp(void *jmp_buf_ptr);
int pvm_install_signal_handlers(void (*handler)(int, pvm_siginfo_t *, void *));

#endif