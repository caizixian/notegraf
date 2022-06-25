CREATE TABLE note
(
    id uuid NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE revision
(
    revision                 uuid        NOT NULL,
    PRIMARY KEY (revision),
    id                       uuid        NOT NULL,
    FOREIGN KEY (id) REFERENCES note (id),
    title                    text        NOT NULL,
    note_inner               text        NOT NULL,
    text_searchable          tsvector GENERATED ALWAYS AS (to_tsvector('english', title || ' ' || note_inner)) STORED,
    parent                   uuid,
    FOREIGN KEY (parent) REFERENCES note (id),
    prev                     uuid UNIQUE,
    FOREIGN KEY (prev) REFERENCES note (id),
    referents                uuid[] NOT NULL,
    metadata_schema_version  bigint      NOT NULL,
    metadata_created_at      timestamptz NOT NULL,
    metadata_modified_at     timestamptz NOT NULL,
    metadata_tags            text[] NOT NULL,
    metadata_custom_metadata jsonb       NOT NULL
);

CREATE INDEX revision_idx_revision ON revision USING HASH (revision);
CREATE INDEX revision_idx_id ON revision USING HASH (id);
CREATE INDEX revision_idx_text_searchable ON revision USING GIN (text_searchable);
CREATE INDEX revision_idx_parent ON revision USING HASH (parent);
CREATE INDEX revision_idx_prev ON revision USING HASH (prev);
CREATE INDEX revision_idx_referents ON revision USING GIN (referents);

CREATE TABLE current_revision
(
    id               uuid NOT NULL UNIQUE,
    FOREIGN KEY (id) REFERENCES note (id),
    current_revision uuid NOT NULL UNIQUE,
    FOREIGN KEY (current_revision) REFERENCES revision (revision)
);

CREATE INDEX current_revision_idx_id ON current_revision USING HASH (id);
CREATE INDEX current_revision_idx_current_revision ON current_revision USING HASH (current_revision);
