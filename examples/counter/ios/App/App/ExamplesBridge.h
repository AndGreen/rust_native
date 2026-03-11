#import <stdbool.h>

bool mf_app_start(float width, float height, float top, float right, float bottom, float left);
void mf_app_tick(void);
void mf_app_resize(float width, float height, float top, float right, float bottom, float left);
bool mf_dev_renderer_apply_message(const char *json);
const char *mf_dev_renderer_take_events_json(void);
void mf_dev_renderer_clear_events_json(void);
void mf_dev_renderer_reset(void);
