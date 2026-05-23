CREATE TABLE IF NOT EXISTS stats (
    device_id TEXT NOT NULL,
    model TEXT,
    version_raw TEXT,
    country TEXT,
    carrier TEXT,
    carrier_id TEXT,
    submit_time timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    version TEXT,
    official boolean,

    CONSTRAINT stats_pkey PRIMARY KEY (device_id)
);

CREATE INDEX IF NOT EXISTS stats_model ON stats(model);
CREATE INDEX IF NOT EXISTS stats_version ON stats(version);
CREATE INDEX IF NOT EXISTS stats_country ON stats(country);
CREATE INDEX IF NOT EXISTS stats_carrier ON stats(carrier);


CREATE TABLE IF NOT EXISTS banned (
    version TEXT,
    model TEXT,
    note TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS banned_version ON banned(version);
CREATE UNIQUE INDEX IF NOT EXISTS banned_model ON banned(model);
