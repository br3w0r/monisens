create type device_init_state as enum ('DEVICE', 'SENSORS');

create table device (
    id integer PRIMARY KEY,
    name text not null constraint name_len CHECK (char_length(name) <= 255),
    display_name text not null constraint display_name_len CHECK (char_length(name) <= 255),
    module_dir text not null constraint module_dir_len CHECK (char_length(module_dir) <= 700),
    data_dir text not null constraint data_dir_len CHECK (char_length(data_dir) <= 700),
    init_state device_init_state not null
);
