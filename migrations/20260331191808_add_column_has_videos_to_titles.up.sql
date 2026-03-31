ALTER TABLE titles ADD COLUMN has_videos boolean DEFAULT false;

CREATE INDEX index_titles_with_videos_on_id ON titles USING btree (id) WHERE has_videos IS true;
