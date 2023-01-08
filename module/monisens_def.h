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

// --------------------------- Конфигурация устройства ---------------------------

typedef enum
{
    // Список поддерживаемых типов. Пока не полный
    DeviceConfInfoEntryTypeSection, // DeviceConfInfo
    DeviceConfInfoEntryTypeString,
    DeviceConfInfoEntryTypeInt,
    DeviceConfInfoEntryTypeIntRange,
    DeviceConfInfoEntryTypeFloat,
    DeviceConfInfoEntryTypeFloatRange,
    DeviceConfInfoEntryTypeJSON,
    DeviceConfInfoEntryTypeChoiceList,
} DeviceConfInfoEntryType;

typedef struct
{
    bool required;
    char *def;

    int32_t *min_len;
    int32_t *max_len;
    char *match_regex;
} DeviceConfInfoEntryString;

typedef struct
{
    bool required;
    int32_t *def;

    int32_t *lt;
    int32_t *gt;
    int32_t *neq;
} DeviceConfInfoEntryInt;

typedef struct
{
    bool required;
    int32_t *def_from;
    int32_t *def_to;

    int32_t min;
    int32_t max;
} DeviceConfInfoEntryIntRange;

typedef struct
{
    bool required;
    float *def;

    float *lt;
    float *gt;
    float *neq;
} DeviceConfInfoEntryFloat;

typedef struct
{
    bool required;
    float *def_from;
    float *def_to;

    float min;
    float max;
} DeviceConfInfoEntryFloatRange;

typedef struct
{
    bool required;
    char *def;
} DeviceConfInfoEntryJSON;

typedef struct
{
    bool required;
    int32_t *def;

    char **choices;
    int32_t chioces_len;
} DeviceConfInfoEntryChoiceList;

typedef struct
{
    char *name;
    DeviceConfInfoEntryType typ; // Тип настройки
    void *data;                  // Данные для настройки (заранее прописанные структуры)
} DeviceConfInfoEntry;

typedef struct
{
    DeviceConfInfoEntry *device_confs;
    int32_t device_confs_len;
} DeviceConfInfo;

typedef void (*device_conf_info_callback)(void *obj, DeviceConfInfo *info);

// ---------------------------- Процесс работы модуля ----------------------------

typedef uint8_t (*mod_version_fn)();

typedef struct
{
    void (*init)(void **handler);
    void (*obtain_device_info)(void *handler, void *obj, device_info_callback callback);
    void (*destroy)(void *handler);
    uint8_t (*connect_device)(void *handler, DeviceConnectConf *connect_info);
    void (*obtain_device_conf_info)(void *handler, void *obj, device_conf_info_callback callback);
} Functions;

typedef Functions (*functions_fn)();
