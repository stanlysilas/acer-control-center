#ifndef ACERCONTROL_HARDWARE_H
#define ACERCONTROL_HARDWARE_H

#include "context.h"

#ifdef __cplusplus
extern "C" {
#endif

#define ACER_HARDWARE_STRING_SIZE 128

typedef enum {
    ACER_HARDWARE_UNKNOWN = 0,
    ACER_HARDWARE_CLEVO_DCHU = 1,
    ACER_HARDWARE_ASPIRE_7 = 2
} AcerHardwarePlatform;

typedef struct {
    int supported;
    AcerHardwarePlatform platform;
    char vendor[ACER_HARDWARE_STRING_SIZE];
    char product_name[ACER_HARDWARE_STRING_SIZE];
    char board_name[ACER_HARDWARE_STRING_SIZE];
    char board_vendor[ACER_HARDWARE_STRING_SIZE];
    char chassis_vendor[ACER_HARDWARE_STRING_SIZE];
    char interface_name[32];
} AcerHardwareInfo;

int acer_hardware_detect(
    AcerContext *ctx,
    AcerHardwareInfo *info);

int acer_hardware_is_supported(
    AcerContext *ctx);

const char *acer_hardware_platform_name(
    AcerHardwarePlatform platform);

#ifdef __cplusplus
}
#endif

#endif
