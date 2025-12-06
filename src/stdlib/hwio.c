/**
 * Hardware I/O Library for Wheel
 * Direct port I/O access for bare metal programming
 */

#include <stdint.h>

/**
 * Read from I/O port
 * @param port: port number
 * @return: value read from port
 */
uint8_t io_read_port_8(uint16_t port) {
    uint8_t result;
    asm("inb %1, %0" : "=a" (result) : "d" (port));
    return result;
}

/**
 * Read 16-bit from I/O port
 */
uint16_t io_read_port_16(uint16_t port) {
    uint16_t result;
    asm("inw %1, %0" : "=a" (result) : "d" (port));
    return result;
}

/**
 * Read 32-bit from I/O port
 */
uint32_t io_read_port_32(uint16_t port) {
    uint32_t result;
    asm("inl %1, %0" : "=a" (result) : "d" (port));
    return result;
}

/**
 * Write to I/O port
 * @param port: port number
 * @param value: 8-bit value to write
 */
void io_write_port_8(uint16_t port, uint8_t value) {
    asm("outb %0, %1" : : "a" (value), "d" (port));
}

/**
 * Write 16-bit to I/O port
 */
void io_write_port_16(uint16_t port, uint16_t value) {
    asm("outw %0, %1" : : "a" (value), "d" (port));
}

/**
 * Write 32-bit to I/O port
 */
void io_write_port_32(uint16_t port, uint32_t value) {
    asm("outl %0, %1" : : "a" (value), "d" (port));
}

/**
 * Generic port I/O read (default 32-bit)
 */
long io_read_port(int port) {
    return (long)io_read_port_32((uint16_t)port);
}

/**
 * Generic port I/O write (default 32-bit)
 */
void io_write_port(int port, long value) {
    io_write_port_32((uint16_t)port, (uint32_t)value);
}

/**
 * Enable interrupts
 */
void io_enable_interrupts() {
    asm("sti");
}

/**
 * Disable interrupts
 */
void io_disable_interrupts() {
    asm("cli");
}

/**
 * Hardware breakpoint / Debug break
 */
void io_break() {
    asm("int $3");
}

/**
 * CPU halt
 */
void io_halt() {
    asm("hlt");
}

/**
 * NOP (no operation)
 */
void io_nop() {
    asm("nop");
}
