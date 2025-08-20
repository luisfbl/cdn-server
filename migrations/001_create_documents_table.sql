CREATE
EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE documents
(
    id         UUID PRIMARY KEY         DEFAULT uuid_generate_v4(),
    hash       VARCHAR(64) NOT NULL,
    file_path  TEXT        NOT NULL,
    file_size  INTEGER     NOT NULL,
    mime_type  VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE document_descriptions
(
    id          UUID PRIMARY KEY         DEFAULT uuid_generate_v4(),
    document_id UUID NOT NULL REFERENCES documents (id) ON DELETE CASCADE,
    description TEXT NOT NULL,
    created_at  TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_documents_hash ON documents (hash);
CREATE INDEX idx_document_descriptions_document_id ON document_descriptions (document_id);