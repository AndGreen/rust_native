#import <stdbool.h>
#import <stdint.h>

bool mf_examples_start(
    uint32_t example_id,
    float width,
    float height,
    float top,
    float right,
    float bottom,
    float left
);
void mf_examples_tick(void);
void mf_examples_resize(float width, float height, float top, float right, float bottom, float left);
