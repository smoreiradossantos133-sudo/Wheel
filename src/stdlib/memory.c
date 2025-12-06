/**
 * Memory Management Library for Wheel
 * Simple heap allocator with malloc/free
 */

#include <stddef.h>
#include <string.h>

#define HEAP_SIZE 1024 * 1024  // 1MB heap
#define MIN_BLOCK_SIZE 32

typedef struct MemBlock {
    size_t size;
    int is_free;
    struct MemBlock* next;
} MemBlock;

static char heap[HEAP_SIZE];
static MemBlock* heap_start = NULL;
static size_t heap_used = 0;

void mem_init() {
    if (heap_start == NULL) {
        heap_start = (MemBlock*)heap;
        heap_start->size = HEAP_SIZE - sizeof(MemBlock);
        heap_start->is_free = 1;
        heap_start->next = NULL;
        heap_used = sizeof(MemBlock);
    }
}

void* mem_alloc(size_t size) {
    mem_init();
    
    if (size == 0) return NULL;
    
    // Find first free block
    MemBlock* current = heap_start;
    while (current != NULL) {
        if (current->is_free && current->size >= size) {
            // Split block if it's larger than needed
            if (current->size > size + sizeof(MemBlock) + MIN_BLOCK_SIZE) {
                MemBlock* new_block = (MemBlock*)((char*)current + sizeof(MemBlock) + size);
                new_block->size = current->size - size - sizeof(MemBlock);
                new_block->is_free = 1;
                new_block->next = current->next;
                
                current->size = size;
                current->next = new_block;
            }
            
            current->is_free = 0;
            return (void*)((char*)current + sizeof(MemBlock));
        }
        current = current->next;
    }
    
    return NULL;  // Not enough memory
}

void mem_free(void* ptr) {
    if (ptr == NULL) return;
    
    MemBlock* block = (MemBlock*)((char*)ptr - sizeof(MemBlock));
    block->is_free = 1;
    
    // Merge with next block if it's free
    if (block->next != NULL && block->next->is_free) {
        block->size += sizeof(MemBlock) + block->next->size;
        block->next = block->next->next;
    }
}

size_t mem_get_used() {
    size_t used = 0;
    MemBlock* current = heap_start;
    while (current != NULL) {
        if (!current->is_free) {
            used += sizeof(MemBlock) + current->size;
        }
        current = current->next;
    }
    return used;
}

size_t mem_get_free() {
    size_t free = 0;
    MemBlock* current = heap_start;
    while (current != NULL) {
        if (current->is_free) {
            free += sizeof(MemBlock) + current->size;
        }
        current = current->next;
    }
    return free;
}

// Aliases for compatibility
void* malloc(size_t size) {
    return mem_alloc(size);
}

void free(void* ptr) {
    mem_free(ptr);
}
