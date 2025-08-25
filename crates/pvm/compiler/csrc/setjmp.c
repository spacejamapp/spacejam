#include "setjmp.h"
#include <stdlib.h>

bool pvm_setjmp(void **buf_storage, bool (*body)(void *, void *), void *payload, void *callee __attribute__((unused))) {
    // Allocate jmp_buf on heap to ensure it remains valid
    platform_jmp_buf *buf = (platform_jmp_buf *)malloc(sizeof(platform_jmp_buf));
    if (buf == NULL) {
        return false;
    }
    
    if (platform_setjmp(*buf) != 0) {
        free(buf);
        return false;
    }
    *buf_storage = buf;
    bool result = body(payload, buf);
    free(buf);
    return result;
}

void pvm_longjmp(void *jmp_buf_ptr) {
    platform_jmp_buf *buf = (platform_jmp_buf *)jmp_buf_ptr;
    platform_longjmp(*buf, 1);
}

int pvm_install_sigsegv_handler(void (*handler)(int, siginfo_t*, void*)) {
    struct sigaction sa = {0};
    sa.sa_sigaction = handler;
    sa.sa_flags = SA_SIGINFO | SA_NODEFER;  // SA_NODEFER allows the signal to be received again
    sigemptyset(&sa.sa_mask);
    return sigaction(SIGSEGV, &sa, NULL);
}