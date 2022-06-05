CREATE TABLE gallery_item (
    title VARCHAR,
    tags VARCHAR NOT NULL,
    created_on TIMESTAMP NOT NULL,
    -- json object
    exif_info VARCHAR NOT NULL
);