#include <stdint.h>
#include <stdbool.h>

// Метод инициализации обработчика модуля. Обработчик (handler) - это любая структура,
// используемая внутри модуля, описываемая разработчиком этого модуля и
// содержащая всю необходимую информацию для корректной работы.
// Память для обработчика выделяется и управляется внутри модуля.
// Для правильного освобождения памяти применяется функция destroy.
void init(void **handler);

typedef enum
{
    ConnParamBool,
    ConnParamInt,
    ConnParamFloat,
    ConnParamString
} ConnParamType;

typedef struct ConnParamConf
{
    char *name;
    ConnParamType typ;
} ConnParamConf;

typedef struct DeviceInfo
{
    ConnParamConf *connection_params;
    int32_t connection_params_len;
} DeviceInfo;

typedef void (*device_info_callback)(void *, DeviceInfo *);

// Функция получения параметров подключения. Она вызывает `callback` из аргумента
// и пердоставляет ему доступ к параметрам. `callback` должен скопировать значения
// из указателя на параметры подключения.
void obtain_device_info(void *handler, void *obj, device_info_callback callback);

typedef struct ConnParam
{
    char *name;
    char *value; // Значение параметра подключения, закодированное в строку
} ConnParam;

typedef struct DeviceConnectInfo
{
    ConnParam *connection_params;
    int32_t connection_params_len;
} DeviceConnectInfo;

// Функция подключения к устройству.
// Возвращает коды ошибок:
//   - 0 - успех,
//   - 1 - подключение неудачно,
//   - 2 - неверные параметры.
// Внутри этой функции модуль благодаря коммуникации с устройством может
// определить, какие параметры возвращать в функции `obtain_sensor_confs`.
// Также здесь следует выполнять проверку правильности конфигурации устройства.
int8_t connect_device(void *handler, DeviceConnectInfo *connect_info);

// Инетрвал записи показаний датчика в миллисекундах
typedef struct RecordInterval
{
    int32_t min;
    int32_t max;
    int32_t def; // Значение по умолчанию
} RecordInterval;

typedef enum
{
    SensorDataInt,
    SensorDataFloat
} SensorDataType;

// Информация о данных, записиываемых датчиком.
typedef struct SensorData
{
    SensorDataType typ;
    char *definition; // Обозначение (Па, Н, мм и т.д.)
    char *min;        // Минимальное воспринимаемое значение, закодированное в строку
    char *max;        // Максимальное воспринимаемое значение, закодированное в строку
} SensorData;

typedef struct SensorInfo
{
    // Имя, которое будет использоваться для записи в базу.
    // Может включать латинские символы и подчёркивание: `_`.
    char *intern_name;
    char *name;
    RecordInterval record_interval;
    SensorData data;
} SensorInfo;

typedef struct SensorInfos
{
    SensorInfo *sensor_infos;
    int32_t sensor_infos_len;
} SensorInfos;

typedef void (*sensor_infos_callback)(void *, SensorInfos *);

// Функция получения конфигурации сенсоров, подключённых к устройству.
// Работает по тому же принципу, что и `obtain_device_conf`.
// Возвращает ошибку `1`, если потеряна связь с устройством.
// Данные, полученные из этой функции используются для создания таблиц в бд
// и сохраняются в конфигурации программы для дальнейшего изменения настроек записи.
int8_t obtain_sensor_infos(void *handler, void *obj, sensor_infos_callback callback);

typedef struct SensorTolerance
{
    char *min, max;
} SensorTolerance;

typedef struct SensorConf
{
    bool enable;
    int32_t record_interval;
    SensorTolerance tolerance;
} SensorConf;

typedef struct DeviceConf
{
    SensorConf *sensor_confs;
    int32_t sensor_confs_len;
} DeviceConf;

// Конфигурация модуля. Получает конфиг для устройства и путь к папке
// для модуля, чтобы хранить данные.
// Возможные ошибки:
//   - 0 - успех,
//   - 1 - подключение прервано,
//   - 2 - ошибка конфигурации.
//
// После выполнения этой функции без ошибок программа создаст таблицы
// для хранения данных с датчиков устройства, сохранит конфигурацию модуля.
int8_t configure(void *handler, DeviceConf *device_conf, char *data_dir);
