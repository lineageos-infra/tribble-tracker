CREATE TABLE IF NOT EXISTS stats (
    device_id character varying NOT NULL,
    model character varying,
    version_raw character varying,
    country character varying,
    carrier character varying,
    carrier_id character varying,
    submit_time timestamp without time zone DEFAULT now(),
    version character varying GENERATED ALWAYS AS ("substring"((version_raw)::text, '^\d\d\.\d'::text)) STORED,

    CONSTRAINT stats_pkey PRIMARY KEY (device_id)
);

