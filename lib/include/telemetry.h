#ifndef ACERCONTROL_TELEMETRY_H
#define ACERCONTROL_TELEMETRY_H

#include "context.h"

#ifdef __cplusplus
extern "C" {
#endif

int acer_cpu_fan_rpm(AcerContext *ctx, int *rpm);
int acer_gpu_fan_rpm(AcerContext *ctx, int *rpm);
int acer_system_fan_rpm(AcerContext *ctx, int *rpm);
int acer_cpu_temperature(AcerContext *ctx, int *temperature);
int acer_gpu_temperature(AcerContext *ctx, int *temperature);
int acer_fan_max(AcerContext *ctx);
int acer_fan_auto(AcerContext *ctx);

#ifdef __cplusplus
}
#endif

#endif
