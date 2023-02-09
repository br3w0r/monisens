create table device_sensor (
	device_id integer not null references device(id),
	sensor_table_name text
);

create unique index device_sensor_idx on device_sensor(device_id, sensor_table_name);
