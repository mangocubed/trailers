CREATE TABLE title_cast (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    title_id uuid NOT NULL,
    person_id uuid NOT NULL,
    tmdb_credit_id varchar NOT NULL,
    character_name varchar NOT NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_title_cast PRIMARY KEY (id),
    CONSTRAINT fkey_title_cast_to_titles FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE,
    CONSTRAINT fkey_title_cast_to_persons FOREIGN KEY (person_id) REFERENCES persons (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_title_cast_on_title_id_person_id ON title_cast USING btree (title_id, person_id);
CREATE UNIQUE INDEX index_title_cast_on_tmdb_credit_id ON title_cast USING btree (tmdb_credit_id);

SELECT manage_updated_at('title_cast');
SELECT manage_versions('title_cast');

CREATE TYPE title_crew_job AS ENUM ('director');

CREATE TABLE title_crew (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    title_id uuid NOT NULL,
    person_id uuid NOT NULL,
    tmdb_credit_id varchar NOT NULL,
    job title_crew_job NOT NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_title_crew PRIMARY KEY (id),
    CONSTRAINT fkey_title_crew_to_titles FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE,
    CONSTRAINT fkey_title_crew_to_persons FOREIGN KEY (person_id) REFERENCES persons (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_title_crew_on_title_id_person_id ON title_crew USING btree (title_id, person_id);
CREATE UNIQUE INDEX index_title_crew_on_tmdb_credit_id ON title_crew USING btree (tmdb_credit_id);

SELECT manage_updated_at('title_crew');
SELECT manage_versions('title_crew');
