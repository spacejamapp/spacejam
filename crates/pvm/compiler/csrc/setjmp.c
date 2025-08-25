#include "setjmp.h"
#include <stdlib.h>

bool pvm_setjmp(void **buf_storage, bool (*body)(void *, void *), void *payload, void *callee __attribute__((unused))) {
    platform_jmp_buf buf;
    if (platform_setjmp(buf) != 0) {
        return false;
    }
    *buf_storage = &buf;
    return body(payload, &buf);
}

void pvm_longjmp(void *jmp_buf_ptr) {
    platform_jmp_buf *buf = (platform_jmp_buf *)jmp_buf_ptr;
    platform_longjmp(*buf, 1);
}

int pvm_install_sigsegv_handler(void (*handler)(int, siginfo_t*, void*)) {
    struct sigaction sa = {0};
    sa.sa_sigaction = handler;
    sa.sa_flags = SA_SIGINFO;
    sigemptyset(&sa.sa_mask);
    return sigaction(SIGSEGV, &sa, NULL);
}