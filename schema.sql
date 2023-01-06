CREATE TABLE IF NOT EXISTS statistics (
    device_id varchar(60) NOT NULL,
    model varchar(60) NOT NULL,
    version_raw varchar(60) NOT NULL,
    country varchar(60) NOT NULL,
    carrier varchar(60) NOT NULL,
    carrier_id varchar(60) NOT NULL,
    submit_time timestamp default now(),
    version varchar(4) GENERATED ALWAYS AS (substring(version_raw from '^\d\d\.\d'))vi
);

