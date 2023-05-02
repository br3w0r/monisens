#include <stdint.h>
#include <stdbool.h>

// ---------------------------- Инициализация ----------------------------

typedef enum
{
    ConnParamBool,
    ConnParamInt,
    ConnParamFloat,
    ConnParamString,
    ConnParamChoiceList
} ConnParamType;

typedef struct
{
    char **choices;
    int32_t chioces_len;
} ConnParamChoiceListInfo;

typedef struct
{
    char *name;
    ConnParamType typ;
    void *info;
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
    int32_t id; // Уникальный id параметра (для типа `DeviceConfInfoEntryTypeSection` не указывается)
    char *name;
    DeviceConfInfoEntryType typ; // Тип параметра
    void *data;                  // Данные для параметра (заранее прописанные структуры)
} DeviceConfInfoEntry;

typedef struct
{
    DeviceConfInfoEntry *device_confs;
    int32_t device_confs_len;
} DeviceConfInfo;

typedef void (*device_conf_info_callback)(void *obj, DeviceConfInfo *info);

// Типы данных по типам параметров:
// DeviceConfInfoEntryTypeString - char *
// DeviceConfInfoEntryTypeInt - int32_t *
// DeviceConfInfoEntryTypeIntRange - int32_t * => массив длины 2: {min, max}
// DeviceConfInfoEntryTypeFloat - float32_t *
// DeviceConfInfoEntryTypeFloatRange - float32_t * => массив длины 2: {min, max}
// DeviceConfInfoEntryTypeJSON - char *
// DeviceConfInfoEntryTypeChoiceList - int32_t * => индекс выбранного пункта в массиве
// Все параметры могут быть NULL.
typedef struct
{
    int32_t id; // Уникальный id параметра
    void *data; // Значение параметра определённого типа
} DeviceConfEntry;

typedef struct
{
    DeviceConfEntry *confs;
    int32_t confs_len;
} DeviceConf;

typedef enum
{
    SensorDataTypeInt16,
    SensorDataTypeInt32,
    SensorDataTypeInt64,
    SensorDataTypeFloat32,
    SensorDataTypeFloat64,
    SensorDataTypeTimestamp,
    SensorDataTypeString,
    SensorDataTypeJSON
} SensorDataType;

typedef struct
{
    // Название данных. Обязательно должно быть написано в snake_case
    // и быть уникальным для каждого данного в рамках одного сенсора.
    char *name;
    SensorDataType typ; // Тип данных
} SensorDataTypeInfo;

typedef struct
{
    // Название сенсора. Обязательно должно быть написано в snake_case
    // и быть уникальным для каждого сенсора в рамках одного устройства.
    char *name;
    int32_t data_type_infos_len;
    SensorDataTypeInfo *data_type_infos; // Массив типов данных сенсора
} SensorTypeInfo;

typedef struct
{
    int32_t sensor_type_infos_len;
    SensorTypeInfo *sensor_type_infos; // Массив информаций о типах данных сенсоров
} SensorTypeInfos;

typedef void (*sensor_type_infos_callback)(void *obj, SensorTypeInfos *infos);

// ------------------------ ц ------------------------

typedef struct
{
    char *name;

    SensorDataType typ;
    // По типу:
    // SensorDataTypeInt16:     int16_t
    // SensorDataTypeInt32:     int32_t
    // SensorDataTypeInt64:     int64_t
    // SensorDataTypeFloat32:   float32_t
    // SensorDataTypeFloat64:   float64_t
    // SensorDataTypeTimestamp: int64_t
    // SensorDataTypeString:    *char
    // SensorDataTypeJSON:      *char
    void *data;
} SensorMsgData;

typedef struct
{
    char *name;
    SensorMsgData *data;
    int32_t data_len;
} SensorMsg;

typedef enum
{
    MsgCodeInfo,
    MsgCodeWarn,
    MsgCodeError,
} MsgCode;

typedef struct
{
    MsgCode code;
    char *msg;
} CommonMsg;

typedef enum
{
    MessageTypeSensor, // data:SensorMsg
    MessageTypeCommon, // data:CommonMsg
} MessageType;

typedef struct
{
    MessageType typ;
    void *data;
} Message;

typedef void (*handle_msg_func)(void *handler, Message msg_data);

// ---------------------------- Процесс работы модуля ----------------------------

typedef uint8_t (*mod_version_fn)();

typedef struct
{
    void (*init)(void **handler);
    void (*obtain_device_info)(void *handler, void *obj, device_info_callback callback);
    void (*destroy)(void *handler);
    uint8_t (*connect_device)(void *handler, DeviceConnectConf *connect_info);
    void (*obtain_device_conf_info)(void *handler, void *obj, device_conf_info_callback callback);
    uint8_t (*configure_device)(void *handler, DeviceConf *conf);
    uint8_t (*obtain_sensor_type_infos)(void *handler, void *obj, sensor_type_infos_callback callback);
    uint8_t (*start)(void *handler, void *msg_handler, handle_msg_func handle_func);
    uint8_t (*stop)(void *handler);
} Functions;

typedef Functions (*functions_fn)();
