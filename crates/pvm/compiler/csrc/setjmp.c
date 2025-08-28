#include "setjmp.h"

// Basic definitions to avoid system headers
#define NULL ((void *)0)
typedef unsigned long size_t;

// Minimal malloc/free declarations to avoid stdlib.h
extern void *malloc(size_t size);
extern void free(void *ptr);

// Platform-specific sigaction structure definition
#ifdef __APPLE__
struct sigaction
{
    union
    {
        void (*sa_handler)(int);
        void (*sa_sigaction)(int, pvm_siginfo_t *, void *);
    } __sigaction_u;
    unsigned int sa_mask;
    int sa_flags;
};
#define sa_sigaction __sigaction_u.sa_sigaction
#else
struct sigaction
{
    union
    {
        void (*sa_handler)(int);
        void (*sa_sigaction)(int, pvm_siginfo_t *, void *);
    };
    unsigned long sa_mask;
    int sa_flags;
    void (*sa_restorer)(void);
};
#endif

bool pvm_setjmp(void **buf_storage, bool (*body)(void *, void *), void *payload, void *callee __attribute__((unused)))
{
    // Allocate jmp_buf on heap to ensure it remains valid
    platform_jmp_buf *buf = (platform_jmp_buf *)malloc(sizeof(platform_jmp_buf));
    if (buf == NULL)
    {
        return false;
    }

    if (platform_setjmp(*buf) != 0)
    {
        free(buf);
        return false;
    }
    *buf_storage = buf;
    bool result = body(payload, buf);
    free(buf);
    return result;
}

void pvm_longjmp(void *jmp_buf_ptr)
{
    platform_jmp_buf *buf = (platform_jmp_buf *)jmp_buf_ptr;
    platform_longjmp(*buf, 1);
}

// Wasmtime-style multi-signal handler installation
int pvm_install_signal_handlers(void (*handler)(int, pvm_siginfo_t *, void *))
{
    struct sigaction sa;

    // Zero out the structure
    char *p = (char *)&sa;
    for (unsigned int i = 0; i < sizeof(struct sigaction); i++)
    {
        p[i] = 0;
    }

    sa.sa_sigaction = handler;
    sa.sa_flags = PVM_SA_SIGINFO | PVM_SA_NODEFER;
    sigemptyset((void *)&sa.sa_mask);

    // Install SIGSEGV handler (Linux primary signal for memory violations)
    if (sigaction(PVM_SIGSEGV, &sa, (struct sigaction *)0) != 0)
    {
        return -1;
    }

    // Install SIGBUS handler (macOS often sends SIGBUS instead of SIGSEGV)
    if (sigaction(PVM_SIGBUS, &sa, (struct sigaction *)0) != 0)
    {
        return -2;
    }

    // Install SIGTRAP handler (macOS might send SIGTRAP for null pointer access)
    if (sigaction(PVM_SIGTRAP, &sa, (struct sigaction *)0) != 0)
    {
        return -3;
    }

    return 0;
}