CREATE TABLE IF NOT EXISTS stats (
    device_id character varying NOT NULL,
    model character varying,
    version_raw character varying,
    country character varying,
    carrier character varying,
    carrier_id character varying,
    submit_time timestamp without time zone DEFAULT now(),
    version character varying GENERATED ALWAYS AS ("substring"((version_raw)::text, '^\d\d\.\d'::text)) STORED,
    official boolean GENERATED ALWAYS AS (version_raw::text ~ '\d\d\.\d-\d{8}-NIGHTLY-.*'::text) STORED,

    CONSTRAINT stats_pkey PRIMARY KEY (device_id)
);

CREATE INDEX stats_model ON stats(model);
CREATE INDEX stats_version ON stats(version);
CREATE INDEX stats_country ON stats(country);
CREATE INDEX stats_carrier ON stats(carrier);


CREATE TABLE IF NOT EXISTS banned (
    version character varying,
    model character varying,
    note character varying
);