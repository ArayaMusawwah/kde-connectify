#ifndef LINKSY_FFI_H
#define LINKSY_FFI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct LinksyNodeHandle LinksyNodeHandle;

/* Session kinds matching linksy_core::SessionKind */
typedef enum {
    LINKSY_SESSION_SCREEN = 0,
    LINKSY_SESSION_CONTROL = 1,
    LINKSY_SESSION_DISPLAY = 2,
    LINKSY_SESSION_CAMERA = 3,
    LINKSY_SESSION_INPUT = 4,
    LINKSY_SESSION_MICROPHONE = 5,
    LINKSY_SESSION_TERMINAL = 6,
} LinksySessionKind;

/* Session event types */
typedef enum {
    LINKSY_SESSION_EVENT_REQUEST = 0,
    LINKSY_SESSION_EVENT_STARTED = 1,
    LINKSY_SESSION_EVENT_ENDED = 2,
} LinksySessionEventType;

/* Callbacks */
typedef void (*linksy_device_cb)(const char *device_id, const char *name, const char *ip, uint16_t port, int is_lost, void *user_data);
typedef void (*linksy_pair_code_cb)(const char *request_id, const char *device_id, const char *code, int incoming, void *user_data);
typedef void (*linksy_packet_cb)(const char *device_id, const char *packet_type, const char *payload_json, void *user_data);
typedef void (*linksy_session_cb)(const char *session_id, const char *device_id, int session_kind, int event_type, void *user_data);
typedef void (*linksy_media_data_cb)(const char *session_id, const uint8_t *data, size_t len, void *user_data);

/* Node lifecycle */
LinksyNodeHandle *linksy_node_new(const char *config_dir, const char *name);
void linksy_node_free(LinksyNodeHandle *node);

/* Callback registration & discovery */
int linksy_start_discovery(LinksyNodeHandle *node, linksy_device_cb cb, void *user_data);
int linksy_set_pairing_callback(LinksyNodeHandle *node, linksy_pair_code_cb cb, void *user_data);
int linksy_set_packet_callback(LinksyNodeHandle *node, linksy_packet_cb cb, void *user_data);
int linksy_set_session_callback(LinksyNodeHandle *node, linksy_session_cb cb, void *user_data);
int linksy_set_media_data_callback(LinksyNodeHandle *node, linksy_media_data_cb cb, void *user_data);

/* Pairing & messaging */
int linksy_pair_device(LinksyNodeHandle *node, const char *device_id);
int linksy_respond_pairing(LinksyNodeHandle *node, const char *request_id, int accept);
int linksy_send_packet(LinksyNodeHandle *node, const char *device_id, const char *packet_type, const char *payload_json);

/* Sessions & media */
int linksy_request_session(LinksyNodeHandle *node, const char *device_id, int session_kind, char *out_session_id, size_t out_session_id_size);
int linksy_respond_session(LinksyNodeHandle *node, const char *request_id, int accept, uint32_t width, uint32_t height);
int linksy_feed_h264(LinksyNodeHandle *node, const char *session_id, const uint8_t *data, size_t len);
int linksy_feed_pcm(LinksyNodeHandle *node, const char *session_id, const uint8_t *data, size_t len);

/* Advanced Transport (AOA USB, Iroh WAN, Wi-Fi Direct) */
int linksy_enable_away(LinksyNodeHandle *node, int enable, const char *relay_url);
int linksy_set_direct_min(LinksyNodeHandle *node, uint64_t min_bytes);
int linksy_is_connected_over_usb(LinksyNodeHandle *node, const char *device_id);

#ifdef __cplusplus
}
#endif

#endif /* LINKSY_FFI_H */
