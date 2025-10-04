BEGIN;

CREATE TYPE kms.KEY_STATUS AS ENUM ('ACTIVE', 'DECRYPT_ONLY', 'DISABLED');

CREATE TABLE kms.keys (
  id public.ulid NOT NULL,
  status kms.KEY_STATUS NOT NULL,
  encrypted_key BYTEA NOT NULL,
  key_version INT NOT NULL,
  root_key_version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  disabled_at TIMESTAMPTZ,
  PRIMARY KEY (id),
  UNIQUE (key_version)
);

COMMIT;
