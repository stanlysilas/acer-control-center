#include <errno.h>
#include <string.h>

#include "../include/driver.h"
#include "../include/ioctl.h"
#include "../include/hardware.h"
#include "../driver/include/acer_io_ioctl.h"

int return_value;

int acer_driver_version(
    AcerContext *ctx,
    char *buffer,
    unsigned int size)
{
    if (!ctx || !buffer || size == 0)
        return -EINVAL;

    memset(buffer, 0, size);

    if (acer_ioctl_read(ctx, R_MOD_VERSION, buffer) < 0)
        return -errno;

    return 0;
}

int acer_driver_hardware_check(
    AcerContext *ctx,
    int *supported)
{
    int return_value;

    if (!ctx || !supported)
        return -EINVAL;

    return_value = acer_hardware_is_supported(ctx);

    if (return_value < 0)
        return return_value;

    *supported = return_value;

    return 0;
}

int acer_driver_interface(
    AcerContext *ctx,
    char *buffer,
    unsigned int size)
{
    if (!ctx || !buffer || size == 0)
        return -EINVAL;

    memset(buffer, 0, size);

    if (acer_ioctl_read(ctx, R_CL_HW_IF_STR, buffer) < 0)
        return -errno;

    return 0;
}

int acer_driver_abi_major(void)
{
    return ACER_IO_ABI_MAJOR;
}

int acer_driver_abi_minor(void)
{
    return ACER_IO_ABI_MINOR;
}
