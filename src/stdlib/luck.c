// Luck library: random number generation
#include <stdlib.h>
#include <time.h>

static int _luck_seed_initialized = 0;

// Initialize random seed (called once)
void luck_init(void) {
    if (!_luck_seed_initialized) {
        srand((unsigned int)time(NULL));
        _luck_seed_initialized = 1;
    }
}

// Generate random number between 0 and max (inclusive)
long luck_random(long max) {
    luck_init();
    if (max <= 0) return 0;
    return (long)(rand() % (max + 1));
}

// Generate random number between min and max (inclusive)
long luck_random_range(long min, long max) {
    luck_init();
    if (min > max) {
        long tmp = min;
        min = max;
        max = tmp;
    }
    if (min == max) return min;
    return min + (long)(rand() % (max - min + 1));
}
