/**
 * Process Management Library for Wheel
 * Simple process creation and scheduling
 */

#include <unistd.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <sys/resource.h>
#include <signal.h>
#include <stdio.h>

#define MAX_PROCESSES 256

typedef struct {
    pid_t pid;
    int status;
    int is_running;
} Process;

static Process processes[MAX_PROCESSES];
static int process_count = 0;

/**
 * Initialize process management system
 */
void process_init() {
    for (int i = 0; i < MAX_PROCESSES; i++) {
        processes[i].pid = -1;
        processes[i].status = 0;
        processes[i].is_running = 0;
    }
    process_count = 0;
}

/**
 * Create a new child process
 * Returns process ID or -1 on error
 */
long process_create(const char* command) {
    if (process_count >= MAX_PROCESSES) return -1;
    
    pid_t child_pid = fork();
    
    if (child_pid < 0) {
        // Fork failed
        return -1;
    } else if (child_pid == 0) {
        // Child process: execute command via shell
        execl("/bin/sh", "sh", "-c", command, NULL);
        // If execl returns, an error occurred
        _exit(127);
    } else {
        // Parent process: record new child
        processes[process_count].pid = child_pid;
        processes[process_count].status = 0;
        processes[process_count].is_running = 1;
        int handle = process_count;
        process_count++;
        return handle;
    }
}

/**
 * Wait for a process to complete
 * Returns exit status or -1 on error
 */
long process_wait(long pid_handle) {
    if (pid_handle < 0 || pid_handle >= process_count) return -1;
    
    int status;
    pid_t result = waitpid(processes[pid_handle].pid, &status, 0);
    
    if (result < 0) return -1;
    
    processes[pid_handle].is_running = 0;
    processes[pid_handle].status = status;
    
    if (WIFEXITED(status)) {
        return WEXITSTATUS(status);
    }
    return -1;
}

/**
 * Check if process is running
 */
long process_is_running(long pid_handle) {
    if (pid_handle < 0 || pid_handle >= process_count) return -1;
    
    if (!processes[pid_handle].is_running) return 0;
    
    // Check with waitpid WNOHANG to see if process has exited
    int status;
    pid_t result = waitpid(processes[pid_handle].pid, &status, WNOHANG);
    
    if (result == 0) {
        return 1;  // Still running
    } else if (result > 0) {
        processes[pid_handle].is_running = 0;
        processes[pid_handle].status = status;
        return 0;  // Process exited
    }
    
    return -1;  // Error
}

/**
 * Kill process
 */
long process_kill(long pid_handle) {
    if (pid_handle < 0 || pid_handle >= process_count) return -1;
    
    int result = kill(processes[pid_handle].pid, SIGTERM);
    if (result == 0) {
        processes[pid_handle].is_running = 0;
    }
    return result;
}

/**
 * Get process PID
 */
long process_get_pid(long pid_handle) {
    if (pid_handle < 0 || pid_handle >= process_count) return -1;
    return (long)processes[pid_handle].pid;
}

/**
 * Yield CPU to other processes
 */
void process_yield() {
    sched_yield();
}

/**
 * Get current process ID
 */
long process_get_current_pid() {
    return (long)getpid();
}

/**
 * Get parent process ID
 */
long process_get_parent_pid() {
    return (long)getppid();
}

/**
 * Set process priority
 */
long process_set_priority(long pid_handle, int priority) {
    if (pid_handle < 0 || pid_handle >= process_count) return -1;
    
    int result = setpriority(PRIO_PROCESS, processes[pid_handle].pid, priority);
    return result;
}

/**
 * Get process resource usage
 */
long process_get_memory(long pid_handle) {
    if (pid_handle < 0 || pid_handle >= process_count) return -1;
    
    struct rusage usage;
    if (getrusage(RUSAGE_CHILDREN, &usage) < 0) return -1;
    
    return (long)(usage.ru_maxrss);
}

// Include sched_yield header at compile time if available
#ifndef _BSD_SOURCE
#define _BSD_SOURCE
#endif
