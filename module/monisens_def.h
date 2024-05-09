#include <stdint.h>
#include <stdbool.h>

// ---------------------------- Initialization ----------------------------

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
    char *value; // Connection parameter value encoded in a string
} ConnParam;

typedef struct
{
    ConnParam *connection_params;
    int32_t connection_params_len;
} DeviceConnectConf;

// --------------------------- Device configuration ---------------------------

typedef enum
{
    // List of supported types
    DeviceConfInfoEntryTypeSection,    // DeviceConfInfo
    DeviceConfInfoEntryTypeString,     // DeviceConfInfoEntryString
    DeviceConfInfoEntryTypeInt,        // DeviceConfInfoEntryInt
    DeviceConfInfoEntryTypeIntRange,   // DeviceConfInfoEntryIntRange
    DeviceConfInfoEntryTypeFloat,      // DeviceConfInfoEntryFloat
    DeviceConfInfoEntryTypeFloatRange, // DeviceConfInfoEntryFloatRange
    DeviceConfInfoEntryTypeJSON,       // DeviceConfInfoEntryJSON
    DeviceConfInfoEntryTypeChoiceList, // DeviceConfInfoEntryChoiceList
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
    int32_t id; // Unique parameter id (not specified for `DeviceConfInfoEntryTypeSection` type)
    char *name;
    DeviceConfInfoEntryType typ; // Type of the parameter
    void *data;                  // Data for the parameter (pre-defined structures)
} DeviceConfInfoEntry;

typedef struct
{
    DeviceConfInfoEntry *device_confs;
    int32_t device_confs_len;
} DeviceConfInfo;

typedef void (*device_conf_info_callback)(void *obj, DeviceConfInfo *info);

// Data types by parameter types:
//   - DeviceConfInfoEntryTypeString - char *
//   - DeviceConfInfoEntryTypeInt - int32_t *
//   - DeviceConfInfoEntryTypeIntRange - int32_t * => array of length 2: {min, max}
//   - DeviceConfInfoEntryTypeFloat - float32_t *
//   - DeviceConfInfoEntryTypeFloatRange - float32_t * => array of length 2: {min, max}
//   - DeviceConfInfoEntryTypeJSON - char *
//   - DeviceConfInfoEntryTypeChoiceList - int32_t * => index of the selected item in the array
//
// All parameters can be NULL.
typedef struct
{
    int32_t id; // Unique parameter id
    void *data; // Value of a parameter of a certain type
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
