BEGIN;

CREATE TYPE kms.key_status AS ENUM ('ACTIVE', 'DECRYPT_ONLY', 'DISABLED');

CREATE TABLE kms.keys (
  id public.ulid NOT NULL,
  status kms.key_status NOT NULL,
  encrypted_key bytea NOT NULL,
  key_version int NOT NULL,
  root_key_version int NOT NULL,
  created_at timestamptz NOT NULL,
  updated_at timestamptz NOT NULL,
  disabled_at timestamptz,
  PRIMARY KEY (id),
  UNIQUE (key_version)
);

COMMIT;
