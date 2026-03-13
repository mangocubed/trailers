CREATE TABLE title_recommendations (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL,
    title_id uuid NOT NULL,
    relevance bigint NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_title_recommendations PRIMARY KEY (id),
    CONSTRAINT fkey_title_recommendations_to_users FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    CONSTRAINT fkey_title_recommendations_to_titles FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_title_recommendations_on_user_id_title_id ON title_recommendations
USING btree (user_id, title_id);
CREATE INDEX index_title_recommendations_on_relevance ON title_recommendations USING btree (relevance);

SELECT manage_updated_at('title_recommendations');
SELECT manage_versions('title_recommendations');
