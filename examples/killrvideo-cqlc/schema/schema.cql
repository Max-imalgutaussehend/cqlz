-- Real KillrVideo schema (Cassandra 3.x version), copied from https://github.com/KillrVideo/killrvideo-data (Apache-2.0).
CREATE TABLE IF NOT EXISTS user_credentials (
    email text,
    password text,
    userid uuid,
    PRIMARY KEY (email)
);

CREATE TABLE IF NOT EXISTS users (
    userid uuid,
    firstname text,
    lastname text,
    email text,
    created_date timestamp,
    PRIMARY KEY (userid)
);

CREATE TABLE IF NOT EXISTS videos (
    videoid uuid,
    userid uuid,
    name text,
    description text,
    location text,
    location_type int,
    preview_image_location text,
    tags set<text>,
    added_date timestamp,
    PRIMARY KEY (videoid)
);

CREATE TABLE IF NOT EXISTS user_videos (
    userid uuid,
    added_date timestamp,
    videoid uuid,
    name text,
    preview_image_location text,
    PRIMARY KEY (userid, added_date, videoid)
) WITH CLUSTERING ORDER BY (added_date DESC, videoid ASC);

CREATE TABLE IF NOT EXISTS latest_videos (
    yyyymmdd text,
    added_date timestamp,
    videoid uuid,
    userid uuid,
    name text,
    preview_image_location text,
    PRIMARY KEY (yyyymmdd, added_date, videoid)
) WITH CLUSTERING ORDER BY (added_date DESC, videoid ASC);

CREATE TABLE IF NOT EXISTS video_ratings (
    videoid uuid,
    rating_counter counter,
    rating_total counter,
    PRIMARY KEY (videoid)
);

CREATE TABLE IF NOT EXISTS video_ratings_by_user (
    videoid uuid,
    userid uuid,
    rating int,
    PRIMARY KEY (videoid, userid)
);

CREATE TABLE IF NOT EXISTS video_playback_stats (
    videoid uuid,
    views counter,
    PRIMARY KEY (videoid)
);

CREATE TABLE IF NOT EXISTS video_recommendations ( 
    userid uuid,
    added_date timestamp,
    videoid uuid,
    rating float,
    authorid uuid,
    name text,
    preview_image_location text,
    PRIMARY KEY(userid, added_date, videoid)
) WITH CLUSTERING ORDER BY (added_date DESC, videoid ASC);

CREATE TABLE IF NOT EXISTS video_recommendations_by_video (
    videoid uuid,
    userid uuid,
    rating float,
    added_date timestamp STATIC,
    authorid uuid STATIC,
    name text STATIC,
    preview_image_location text STATIC,
    PRIMARY KEY(videoid, userid)
);

CREATE TABLE IF NOT EXISTS videos_by_tag (
    tag text,
    videoid uuid,
    added_date timestamp,
    userid uuid,
    name text,
    preview_image_location text,
    tagged_date timestamp,
    PRIMARY KEY (tag, videoid)
);

CREATE TABLE IF NOT EXISTS tags_by_letter (
    first_letter text,
    tag text,
    PRIMARY KEY (first_letter, tag)
);

CREATE TABLE IF NOT EXISTS comments_by_video (
    videoid uuid,
    commentid timeuuid,
    userid uuid,
    comment text,
    PRIMARY KEY (videoid, commentid)
) WITH CLUSTERING ORDER BY (commentid DESC);

CREATE TABLE IF NOT EXISTS comments_by_user (
    userid uuid,
    commentid timeuuid,
    videoid uuid,
    comment text,
    PRIMARY KEY (userid, commentid)
) WITH CLUSTERING ORDER BY (commentid DESC);
