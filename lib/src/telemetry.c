#include <errno.h>
#include <stdint.h>

#include "../include/ioctl.h"
#include "../include/telemetry.h"
#include "../driver/include/acer_io_ioctl.h"

static int read_dchu_fan_rpm(AcerContext *ctx, unsigned int fan_index, int *rpm)
{
    uint8_t status[ACER_DCHU_STATUS_SIZE] = { 0 };
    unsigned int offset = 0x02 + fan_index * 2;
    uint16_t raw_tach;

    if (!ctx || !rpm || fan_index > 2)
        return -EINVAL;

    if (acer_ioctl_read(ctx, R_CL_DCHU_STATUS, status) < 0)
        return -errno;

    raw_tach = (uint16_t)(((uint16_t)status[offset] << 8) | status[offset + 1]);
    *rpm = raw_tach ? (int)(2156220U / raw_tach) : 0;
    return 0;
}

static int faninfo_to_temperature(int faninfo)
{
    return (faninfo >> 8) & 0xff;
}

int acer_cpu_fan_rpm(AcerContext *ctx, int *rpm)
{
    return read_dchu_fan_rpm(ctx, 0, rpm);
}

int acer_gpu_fan_rpm(AcerContext *ctx, int *rpm)
{
    return read_dchu_fan_rpm(ctx, 1, rpm);
}

int acer_system_fan_rpm(AcerContext *ctx, int *rpm)
{
    return read_dchu_fan_rpm(ctx, 2, rpm);
}

int acer_cpu_temperature(AcerContext *ctx, int *temperature)
{
    int32_t raw = 0;

    if (!ctx || !temperature)
        return -EINVAL;

    if (acer_ioctl_read(ctx, R_CL_FANINFO1, &raw) < 0)
        return -errno;

    *temperature = faninfo_to_temperature(raw);

    return 0;
}

int acer_gpu_temperature(AcerContext *ctx, int *temperature)
{
    int32_t raw = 0;

    if (!ctx || !temperature)
        return -EINVAL;

    if (acer_ioctl_read(ctx, R_CL_FANINFO2, &raw) < 0)
        return -errno;

    *temperature = faninfo_to_temperature(raw);

    return 0;
}

int acer_fan_max(AcerContext *ctx)
{
    int32_t argument = 0x00ffffff;

    if (!ctx)
        return -EINVAL;

    if (acer_ioctl_write(ctx, W_CL_FANSPEED, &argument) < 0)
        return -errno;

    return 0;
}

int acer_fan_auto(AcerContext *ctx)
{
    int32_t argument = 0;

    if (!ctx)
        return -EINVAL;

    if (acer_ioctl_write(ctx, W_CL_FANAUTO, &argument) < 0)
        return -errno;

    return 0;
}
