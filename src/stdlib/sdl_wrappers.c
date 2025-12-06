// SDL2 C wrappers for Wheel
// Compile with: gcc -fPIC -shared -lSDL2 sdl_wrappers.c -o libwheel_sdl.so

#include <SDL2/SDL.h>
#include <stdint.h>

static SDL_Window* wheel_window = NULL;
static SDL_Renderer* wheel_renderer = NULL;

int64_t sdl_init() {
    if (SDL_Init(SDL_INIT_VIDEO) != 0) {
        return 0;
    }
    return 1;
}

int64_t sdl_create_window(int32_t width, int32_t height, int64_t title_ptr) {
    const char* title = (const char*)title_ptr;
    
    wheel_window = SDL_CreateWindow(
        title,
        SDL_WINDOWPOS_CENTERED,
        SDL_WINDOWPOS_CENTERED,
        width,
        height,
        SDL_WINDOW_SHOWN
    );
    
    if (!wheel_window) {
        return 0;
    }
    
    wheel_renderer = SDL_CreateRenderer(wheel_window, -1, SDL_RENDERER_ACCELERATED);
    if (!wheel_renderer) {
        SDL_DestroyWindow(wheel_window);
        wheel_window = NULL;
        return 0;
    }
    
    return 1;
}

int64_t sdl_draw_pixel(int32_t x, int32_t y, uint8_t r, uint8_t g, uint8_t b) {
    if (!wheel_renderer) return 0;
    
    SDL_SetRenderDrawColor(wheel_renderer, r, g, b, 255);
    int result = SDL_RenderDrawPoint(wheel_renderer, x, y);
    return result == 0 ? 1 : 0;
}

int64_t sdl_draw_rect(int32_t x, int32_t y, int32_t w, int32_t h, uint8_t r, uint8_t g, uint8_t b) {
    if (!wheel_renderer) return 0;
    
    SDL_SetRenderDrawColor(wheel_renderer, r, g, b, 255);
    SDL_Rect rect = {x, y, w, h};
    int result = SDL_RenderFillRect(wheel_renderer, &rect);
    return result == 0 ? 1 : 0;
}

int64_t sdl_clear(uint8_t r, uint8_t g, uint8_t b) {
    if (!wheel_renderer) return 0;
    
    SDL_SetRenderDrawColor(wheel_renderer, r, g, b, 255);
    int result = SDL_RenderClear(wheel_renderer);
    return result == 0 ? 1 : 0;
}

int64_t sdl_present() {
    if (!wheel_renderer) return 0;
    
    SDL_RenderPresent(wheel_renderer);
    return 1;
}

int64_t sdl_destroy_window() {
    if (wheel_renderer) {
        SDL_DestroyRenderer(wheel_renderer);
        wheel_renderer = NULL;
    }
    if (wheel_window) {
        SDL_DestroyWindow(wheel_window);
        wheel_window = NULL;
    }
    return 1;
}

int64_t sdl_quit() {
    sdl_destroy_window();
    SDL_Quit();
    return 1;
}

// Poll for SDL events and return key codes or -1 for quit, 0 for none
int64_t sdl_poll_event() {
    if (!wheel_renderer && !wheel_window) return 0;
    SDL_Event e;
    while (SDL_PollEvent(&e)) {
        // log events for debugging
        FILE *f = fopen("/workspaces/Wheel/sdl_events.log", "a");
        if (f) {
            fprintf(f, "event type: %d\n", e.type);
        }
        if (f) fclose(f);
        if (e.type == SDL_QUIT) {
            FILE *f2 = fopen("/workspaces/Wheel/sdl_events.log", "a");
            if (f2) { fprintf(f2, "SDL_QUIT\n"); fclose(f2); }
            return -1;
        }
        if (e.type == SDL_KEYDOWN) {
            FILE *f3 = fopen("/workspaces/Wheel/sdl_events.log", "a");
            if (f3) { fprintf(f3, "KEYDOWN: keysym=%d\n", e.key.keysym.sym); fclose(f3); }
            SDL_Keycode k = e.key.keysym.sym;
            switch (k) {
                case SDLK_w: return 1; // up
                case SDLK_a: return 2; // left
                case SDLK_s: return 3; // down
                case SDLK_d: return 4; // right
                case SDLK_UP: return 1;
                case SDLK_LEFT: return 2;
                case SDLK_DOWN: return 3;
                case SDLK_RIGHT: return 4;
                case SDLK_ESCAPE: return -2; // escape
                default: return 0;
            }
        }
    }
    return 0;
}

// Delay in milliseconds
int64_t sdl_delay(int64_t ms) {
    if (ms <= 0) return 0;
    SDL_Delay((Uint32)ms);
    return 1;
}
