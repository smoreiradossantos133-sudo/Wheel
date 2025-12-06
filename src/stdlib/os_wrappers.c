// OS/Syscalls C wrappers for Wheel
// Provides process and timing operations

#include <unistd.h>
#include <sys/types.h>
#include <time.h>
#include <stdint.h>

// These wrapper functions provide the interface expected by Wheel
// The actual system functions are called internally

int64_t wheel_getpid() {
    pid_t pid = getpid();
    return (int64_t)pid;
}

int64_t wheel_time_now() {
    time_t t = time(NULL);
    return (int64_t)t;
}

int64_t wheel_sleep(int64_t seconds) {
    unsigned int sec = (unsigned int)seconds;
    sleep(sec);
    return 1;
}
