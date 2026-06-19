-- SPDX-FileCopyrightText: 2026 The LineageOS Project
--
-- SPDX-License-Identifier: Apache-2.0

ALTER TABLE stats RENAME TO stats_old;

CREATE TABLE stats (
    device_id TEXT NOT NULL,
    model TEXT NOT NULL,
    version_raw TEXT NOT NULL,
    country TEXT NOT NULL COLLATE NOCASE,
    carrier TEXT NOT NULL COLLATE NOCASE,
    carrier_id TEXT NOT NULL,
    submit_time TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    version TEXT NOT NULL,
    official BOOLEAN NOT NULL,
    CONSTRAINT stats_pkey PRIMARY KEY (device_id)
);

INSERT INTO stats SELECT * from stats_old;
DROP TABLE stats_old;

CREATE INDEX stats_model ON stats (model);
CREATE INDEX stats_version ON stats (version);
CREATE INDEX stats_country ON stats (country);
CREATE INDEX stats_carrier ON stats (carrier);
