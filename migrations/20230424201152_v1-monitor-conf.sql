create type monitor_type as enum ('LOG');

create table monitor_conf (
    id serial primary key,
    device_id integer not null references device(id),
    sensor text not null, -- iss-96: consider referencing device_sensor
    typ monitor_type not null,
    config jsonb not null
);
