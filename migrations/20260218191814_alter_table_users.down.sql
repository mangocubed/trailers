ALTER TABLE users
DROP COLUMN identity_user_id,
ADD COLUMN username citext NOT NULL,
ADD COLUMN email citext NOT NULL,
ADD COLUMN encrypted_password varchar NOT NULL,
ADD COLUMN full_name varchar(255) NOT NULL,
ADD COLUMN display_name varchar(255) NOT NULL,
ADD COLUMN birthdate date NOT NULL,
ADD COLUMN language_code varchar(2) NOT NULL DEFAULT 'en',
ADD COLUMN country_code varchar(2) NOT NULL;
