#ifndef PVM_SETJMP_H
#define PVM_SETJMP_H

#include <setjmp.h>
#include <signal.h>
#include <stdbool.h>
#include <stdint.h>

// Platform-specific setjmp/longjmp selection following Wasmtime's approach
#if (defined(__GNUC__) && !defined(__clang__))
#define PVM_GCC 1
#endif

#ifdef _WIN32
#define platform_setjmp(buf) setjmp(buf)
#define platform_longjmp(buf, arg) longjmp(buf, arg)
typedef jmp_buf platform_jmp_buf;

#elif defined(PVM_GCC) || defined(__x86_64__)
#define platform_setjmp(buf) __builtin_setjmp(buf)
#define platform_longjmp(buf, arg) __builtin_longjmp(buf, arg)
typedef void *platform_jmp_buf[5];

#else
#define platform_setjmp(buf) sigsetjmp(buf, 0)
#define platform_longjmp(buf, arg) siglongjmp(buf, arg)
typedef sigjmp_buf platform_jmp_buf;
#endif

// PVM trap handling API
bool pvm_setjmp(void **buf_storage, bool (*body)(void *, void *), void *payload, void *callee);
void pvm_longjmp(void *jmp_buf_ptr);
int pvm_install_sigsegv_handler(void (*handler)(int, siginfo_t*, void*));

#endif