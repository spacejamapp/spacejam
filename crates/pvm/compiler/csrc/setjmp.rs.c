#include "setjmp.rs.h"
#include <stdlib.h>

bool setjmp_rs(void **buf_storage, bool (*body)(void *, void *), void *payload, void *callee __attribute__((unused)))
{
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

void longjmp_rs(void *jmp_buf_ptr)
{
    platform_jmp_buf *buf = (platform_jmp_buf *)jmp_buf_ptr;
    platform_longjmp(*buf, 1);
}

#ifdef __APPLE__
int install_signal_handlers(void (*handler)(int, pvm_siginfo_t *, void *))
{
    struct sigaction sa;

    // Zero out the structure
    char *p = (char *)&sa;
    for (unsigned int i = 0; i < sizeof(struct sigaction); i++)
    {
        p[i] = 0;
    }

    sa.sa_sigaction = (void (*)(int, siginfo_t *, void *))handler;
    sa.sa_flags = PVM_SA_SIGINFO | PVM_SA_NODEFER;
    sigemptyset(&sa.sa_mask);

    // Install SIGSEGV handler (primary signal for memory violations)
    if (sigaction(PVM_SIGSEGV, &sa, NULL) != 0)
    {
        return -1;
    }

    // Install SIGBUS handler (macOS often sends SIGBUS instead of SIGSEGV)
    if (sigaction(PVM_SIGBUS, &sa, NULL) != 0)
    {
        return -2;
    }

    // Install SIGTRAP handler (macOS might send SIGTRAP for null pointer access)
    if (sigaction(PVM_SIGTRAP, &sa, NULL) != 0)
    {
        return -3;
    }

    return 0;
}
#else
// Linux signal handler (currently just SIGSEGV, expandable for future signals)
int install_signal_handlers(void (*handler)(int, pvm_siginfo_t *, void *))
{
    struct sigaction sa = {0};
    sa.sa_sigaction = (void (*)(int, siginfo_t *, void *))handler;
    sa.sa_flags = PVM_SA_SIGINFO | PVM_SA_NODEFER;
    sigemptyset(&sa.sa_mask);

    // Install SIGSEGV handler
    if (sigaction(PVM_SIGSEGV, &sa, NULL) != 0)
    {
        return -1;
    }

    return 0;
}
#endif