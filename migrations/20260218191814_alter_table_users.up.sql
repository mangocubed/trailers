ALTER TABLE users
DROP COLUMN username,
DROP COLUMN email,
DROP COLUMN encrypted_password,
DROP COLUMN full_name,
DROP COLUMN display_name,
DROP COLUMN birthdate,
DROP COLUMN language_code,
DROP COLUMN country_code,
ADD COLUMN identity_user_id uuid NOT NULL;

CREATE UNIQUE INDEX index_users_on_identity_user_id ON users USING btree (identity_user_id);
