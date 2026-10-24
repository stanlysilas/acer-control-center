project(
    'libacercontrol',
    'c',
    version: '2.0.0',
    default_options: ['warning_level=3'],
)

inc = include_directories(
    'include',
    '../driver/include',
)

src = files(
    'src/acercontrol.c',
    'src/context.c',
    'src/driver.c',
    'src/ioctl.c',
    'src/telemetry.c',
    'src/hardware.c',
)

libacercontrol = library(
    'acercontrol',
    src,
    include_directories: inc,
    install: true,
    version: meson.project_version(),
    soversion: '0',
)

install_headers(
    'include/acercontrol.h',
    'include/context.h',
    'include/version.h',
    'include/driver.h',
    'include/ioctl.h',
    'include/telemetry.h',
    'include/hardware.h',
    subdir: 'acercontrol',
)
