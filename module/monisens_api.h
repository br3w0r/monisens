#include "monisens_def.h"

// ------------------------------------------------------------------------------------------
// ----------------------------------- Initialization ---------------------------------------
// ------------------------------------------------------------------------------------------

// Function of module handler initialization. Runs at the first initialization
// and at secondary launches of the already initialized module.
// A handler is any structure, used inside a module, described by
// the developer of this module and containing all necessary information
// for correct operation. Memory for a handler is allocated and managed inside
// the module. The destroy function is used to properly free memory.
void init(void **handler, char *data_dir);

// Function for getting connection parameters. It calls `callback` from the argument
// and gives it access to the parameters. The `callback` will copy the values
// from the pointer to internal storage provided by `obj` argument.
void obtain_device_info(void *handler, void *obj, device_info_callback callback);

// Device connection function.
// Returns error codes:
//   - 0 - success,
//   - 1 - connection failed,
//   - 2 - invalid parameters.
// Inside this function, the module, due to communication with the device, can
// determine which parameters to return in the `obtain_sensor_confs` function.
// Also here you should check whether the device configuration is correct.
uint8_t connect_device(void *handler, DeviceConnectConf *connect_conf);

// ------------------------------------------------------------------------------------------
// -------------------------------- Device configuration ------------------------------------
// ------------------------------------------------------------------------------------------

// Obtain parameters for device configuration. Uses the same pattern as `obtain_device_info`.
void obtain_device_conf_info(void *handler, void *obj, device_conf_info_callback callback);

// Device configuration based on parameters from `obtain_device_conf_info`.
// MoniSens guarantees that all device configuration parameters are present in `conf`.
// Returns error codes:
//   - 0 - success,
//   - 1 - connection failed,
//   - 2 - invalid parameters.
uint8_t configure_device(void *handler, DeviceConf *conf);

// ------------------------------------------------------------------------------------------
// ----------------------- Functions to obtain device information  --------------------------
// ------------------------------------------------------------------------------------------

// Get information about data types received from sensors
// Returns error codes:
// - 0 - success,
// - 1 - connection failed.
// The system will return its errors separately if the names of sensors and their data do not pass the
// validation.
uint8_t obtain_sensor_type_infos(void *handler, void *obj, sensor_type_infos_callback callback);

// -------------------------------------------------------------------------------------------
// ----------------------- Функции для коммуникации с устройством -------------------------
// -------------------------------------------------------------------------------------------

// Start the module.
//
// `msg_handler` can be safely sent between threads.
uint8_t start(void *handler, void *msg_handler, handle_msg_func handle_func);

// Stop module operation.
//
// After executing this function, the module must ensure that the `msg_handler`
// and `handle_func` passed in the `start()` call have been removed from memory.
uint8_t stop(void *handler);

// -------------------------------------------------------------------------------------------
// ----------------------- Functions for module' working process -----------------------------
// -------------------------------------------------------------------------------------------

// Function for freeing memory allocated for the handler.
void destroy(void *handler);

// Funciton returning the used version of the header.
// It's made for compatibility with older module versions in the future.
uint8_t mod_version();

// Funciton that returns all module functions
Functions functions();
