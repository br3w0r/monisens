create table device_sensor (
	device_id integer not null references device(id),
	sensor_table_name text
);
