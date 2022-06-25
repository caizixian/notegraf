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

CREATE TABLE current_revision
(
    id               uuid NOT NULL UNIQUE,
    FOREIGN KEY (id) REFERENCES note (id),
    current_revision uuid NOT NULL UNIQUE,
    FOREIGN KEY (current_revision) REFERENCES revision (revision)
);