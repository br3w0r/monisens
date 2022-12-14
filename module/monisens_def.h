#include <stdint.h>
#include <stdbool.h>

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

typedef struct DeviceConnectConf
{
    ConnParam *connection_params;
    int32_t connection_params_len;
} DeviceConnectConf;

// Инетрвал записи показаний датчика в миллисекундах
typedef struct
{
    int32_t min;
    int32_t max;
    int32_t def;
} RecordInterval;

typedef enum
{
    SensorDataInt,
    SensorDataFloat
} SensorDataType;

// Информация о данных, записиываемых датчиком.
typedef struct
{
    SensorDataType typ;
    char *definition; // Обозначение (Па, Н, мм и т.д.)
    char *min;        // Минимальное воспринимаемое значение, закодированное в строку
    char *max;        // Максимальное воспринимаемое значение, закодированное в строку
} SensorDataInfo;

typedef struct
{
    // Имя, которое будет использоваться для записи в базу.
    // Может включать латинские символы и подчёркивание: `_`.
    char *intern_name;
    char *name;
    RecordInterval record_interval;
    SensorDataInfo data;
} SensorInfo;

typedef struct
{
    SensorInfo *sensor_infos;
    int32_t sensor_infos_len;
} SensorInfos;

typedef void (*sensor_infos_callback)(void *, SensorInfos *);

typedef struct
{
    bool enable;
    int32_t record_interval;
} SensorConf;

typedef struct
{
    SensorConf *sensor_confs;
    int32_t sensor_confs_len;
} DeviceConf;

typedef uint8_t (*mod_version_fn)();

typedef struct
{
    void (*init)(void **handler);
    void (*obtain_device_info)(void *handler, void *obj, device_info_callback callback);
    void (*destroy)(void *handler);
    uint8_t (*connect_device) (void *handler, DeviceConnectConf *connect_info);
} Functions;

typedef Functions (*functions_fn)();
