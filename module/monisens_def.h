#include <stdint.h>
#include <stdbool.h>

// ---------------------------- Инициализация ----------------------------

typedef enum
{
    ConnParamBool,
    ConnParamInt,
    ConnParamFloat,
    ConnParamString
} ConnParamType;

typedef struct
{
    char *name;
    ConnParamType typ;
} ConnParamInfo;

typedef struct
{
    ConnParamInfo *connection_params;
    int32_t connection_params_len;
} DeviceConnectInfo;

typedef void (*device_info_callback)(void *, DeviceConnectInfo *);

typedef struct
{
    char *name;
    char *value; // Значение параметра подключения, закодированное в строку
} ConnParam;

typedef struct
{
    ConnParam *connection_params;
    int32_t connection_params_len;
} DeviceConnectConf;

typedef enum
{
    SensorDataInt,
    SensorDataFloat
} SensorDataType;

// ---------------------------- Процесс работы модуля ----------------------------

typedef uint8_t (*mod_version_fn)();

typedef struct
{
    void (*init)(void **handler);
    void (*obtain_device_info)(void *handler, void *obj, device_info_callback callback);
    void (*destroy)(void *handler);
    uint8_t (*connect_device)(void *handler, DeviceConnectConf *connect_info);
} Functions;

typedef Functions (*functions_fn)();
