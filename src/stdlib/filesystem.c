/**
 * Filesystem Library for Wheel
 * Low-level disk I/O operations
 */

#include <stdint.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>

#define SECTOR_SIZE 512

typedef struct {
    int fd;
    int block_size;
    int total_blocks;
} FSHandle;

static FSHandle fs_handles[4];
static int fs_handle_count = 0;

/**
 * Initialize filesystem on a device
 * Returns handle ID or -1 on error
 */
int fs_open(const char* device) {
    if (fs_handle_count >= 4) return -1;
    
    int fd = open(device, O_RDWR);
    if (fd < 0) return -1;
    
    fs_handles[fs_handle_count].fd = fd;
    fs_handles[fs_handle_count].block_size = SECTOR_SIZE;
    fs_handles[fs_handle_count].total_blocks = 0;
    
    return fs_handle_count++;
}

/**
 * Close filesystem handle
 */
void fs_close(int handle) {
    if (handle >= 0 && handle < fs_handle_count) {
        close(fs_handles[handle].fd);
        fs_handles[handle].fd = -1;
    }
}

/**
 * Read block from disk
 * @param handle: filesystem handle
 * @param block_num: block number to read
 * @param buffer: destination buffer (must be at least SECTOR_SIZE bytes)
 * @return: number of bytes read or -1 on error
 */
long fs_read_block(int handle, long block_num, void* buffer) {
    if (handle < 0 || handle >= fs_handle_count) return -1;
    
    int fd = fs_handles[handle].fd;
    if (fd < 0) return -1;
    
    off_t offset = block_num * SECTOR_SIZE;
    if (lseek(fd, offset, SEEK_SET) < 0) return -1;
    
    ssize_t bytes_read = read(fd, buffer, SECTOR_SIZE);
    return (bytes_read < 0) ? -1 : bytes_read;
}

/**
 * Write block to disk
 * @param handle: filesystem handle
 * @param block_num: block number to write
 * @param buffer: source buffer (must be at least SECTOR_SIZE bytes)
 * @return: number of bytes written or -1 on error
 */
long fs_write_block(int handle, long block_num, const void* buffer) {
    if (handle < 0 || handle >= fs_handle_count) return -1;
    
    int fd = fs_handles[handle].fd;
    if (fd < 0) return -1;
    
    off_t offset = block_num * SECTOR_SIZE;
    if (lseek(fd, offset, SEEK_SET) < 0) return -1;
    
    ssize_t bytes_written = write(fd, buffer, SECTOR_SIZE);
    return (bytes_written < 0) ? -1 : bytes_written;
}

/**
 * Get file size
 */
long fs_get_size(int handle) {
    if (handle < 0 || handle >= fs_handle_count) return -1;
    
    int fd = fs_handles[handle].fd;
    if (fd < 0) return -1;
    
    struct stat sb;
    if (fstat(fd, &sb) < 0) return -1;
    
    return sb.st_size;
}

/**
 * Sync filesystem (flush to disk)
 */
void fs_sync(int handle) {
    if (handle >= 0 && handle < fs_handle_count) {
        int fd = fs_handles[handle].fd;
        if (fd >= 0) {
            fsync(fd);
        }
    }
}

/**
 * Set block size for operations
 */
void fs_set_block_size(int handle, int block_size) {
    if (handle >= 0 && handle < fs_handle_count) {
        fs_handles[handle].block_size = block_size;
    }
}

/**
 * Get current block size
 */
int fs_get_block_size(int handle) {
    if (handle >= 0 && handle < fs_handle_count) {
        return fs_handles[handle].block_size;
    }
    return -1;
}
