#include <stdint.h>
#include <stdbool.h>

// ---------------------------- Configuration -----------------------------

typedef enum
{
    // List of supported types
    ConfInfoEntryTypeSection,    // ConfInfo
    ConfInfoEntryTypeString,     // ConfInfoEntryString
    ConfInfoEntryTypeInt,        // ConfInfoEntryInt
    ConfInfoEntryTypeIntRange,   // ConfInfoEntryIntRange
    ConfInfoEntryTypeFloat,      // ConfInfoEntryFloat
    ConfInfoEntryTypeFloatRange, // ConfInfoEntryFloatRange
    ConfInfoEntryTypeJSON,       // ConfInfoEntryJSON
    ConfInfoEntryTypeChoiceList, // ConfInfoEntryChoiceList
} ConfInfoEntryType;

typedef struct
{
    bool required;
    char *def;

    int32_t *min_len;
    int32_t *max_len;
    char *match_regex;
} ConfInfoEntryString;

typedef struct
{
    bool required;
    int32_t *def;

    int32_t *lt;
    int32_t *gt;
    int32_t *neq;
} ConfInfoEntryInt;

typedef struct
{
    bool required;
    int32_t *def_from;
    int32_t *def_to;

    int32_t min;
    int32_t max;
} ConfInfoEntryIntRange;

typedef struct
{
    bool required;
    float *def;

    float *lt;
    float *gt;
    float *neq;
} ConfInfoEntryFloat;

typedef struct
{
    bool required;
    float *def_from;
    float *def_to;

    float min;
    float max;
} ConfInfoEntryFloatRange;

typedef struct
{
    bool required;
    char *def;
} ConfInfoEntryJSON;

typedef struct
{
    bool required;
    int32_t *def;

    char **choices;
    int32_t chioces_len;
} ConfInfoEntryChoiceList;

typedef struct
{
    int32_t id; // Unique parameter id (not specified for `ConfInfoEntryTypeSection` type)
    char *name;
    ConfInfoEntryType typ; // Type of the parameter
    void *data;                  // Data for the parameter (pre-defined structures)
} ConfInfoEntry;

typedef struct
{
    ConfInfoEntry *confs;
    int32_t confs_len;
} ConfInfo;

typedef void (*device_conn_info_callback)(void *, ConfInfo *);

typedef void (*device_conf_info_callback)(void *obj, ConfInfo *info);

// Data types by parameter types:
//   - ConfInfoEntryTypeString - char *
//   - ConfInfoEntryTypeInt - int32_t *
//   - ConfInfoEntryTypeIntRange - int32_t * => array of length 2: {min, max}
//   - ConfInfoEntryTypeFloat - float32_t *
//   - ConfInfoEntryTypeFloatRange - float32_t * => array of length 2: {min, max}
//   - ConfInfoEntryTypeJSON - char *
//   - ConfInfoEntryTypeChoiceList - int32_t * => index of the selected item in the array
//
// All parameters can be NULL.
typedef struct
{
    int32_t id; // Unique parameter id
    void *data; // Value of a parameter of a certain type
} ConfEntry;

typedef struct
{
    ConfEntry *confs;
    int32_t confs_len;
} Conf;

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
    // Data name. Must be written in snake_case
    // and be unique for each data within one sensor.
    char *name;
    SensorDataType typ; // Тип данных
} SensorDataTypeInfo;

typedef struct
{
    // Sensor name. Must be written in snake_case
    // and be unique for each sensor within one device.
    char *name;
    int32_t data_type_infos_len;
    SensorDataTypeInfo *data_type_infos; // Sensor data type array
} SensorTypeInfo;

typedef struct
{
    int32_t sensor_type_infos_len;
    SensorTypeInfo *sensor_type_infos; // Array of infos about sensor data types
} SensorTypeInfos;

typedef void (*sensor_type_infos_callback)(void *obj, SensorTypeInfos *infos);

// ------------------------ Device communication ------------------------

// Entity of single data from the sensor
typedef struct
{
    char *name;

    SensorDataType typ;
    // By type:
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

// All data from a single sensor
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

// Logging message from module
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

// Message from the module
typedef struct
{
    MessageType typ;
    void *data;
} Message;

// Function for handling messages from the module
typedef void (*handle_msg_func)(void *handler, Message msg_data);

// ---------------------------- Module's working process ----------------------------

typedef uint8_t (*mod_version_fn)();

// All functions defined in monisens_api.h
typedef struct
{
    void (*init)(void **handler, char *data_dir);
    void (*obtain_device_conn_info)(void *handler, void *obj, device_conn_info_callback callback);
    void (*destroy)(void *handler);
    uint8_t (*connect_device)(void *handler, Conf *connect_conf);
    void (*obtain_device_conf_info)(void *handler, void *obj, device_conf_info_callback callback);
    uint8_t (*configure_device)(void *handler, Conf *device_conf);
    uint8_t (*obtain_sensor_type_infos)(void *handler, void *obj, sensor_type_infos_callback callback);
    uint8_t (*start)(void *handler, void *msg_handler, handle_msg_func handle_func);
    uint8_t (*stop)(void *handler);
} Functions;

typedef Functions (*functions_fn)();
