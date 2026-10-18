-- Queries written for this example, following the access patterns that the KillrVideo schema comments describe.

-- name: GetUser :one
SELECT userid, firstname, lastname, email, created_date
FROM users WHERE userid = ?;

-- name: GetCredentials :one
SELECT email, password, userid FROM user_credentials WHERE email = ?;

-- name: CreateUser :exec
INSERT INTO users (userid, firstname, lastname, email, created_date) VALUES (?, ?, ?, ?, ?);

-- name: GetVideo :one
SELECT videoid, userid, name, description, location, location_type, preview_image_location, tags, added_date
FROM videos WHERE videoid = ?;

-- name: AddVideo :exec
INSERT INTO videos (videoid, userid, name, description, location, location_type, preview_image_location, tags, added_date)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);

-- name: AddUserVideo :exec
INSERT INTO user_videos (userid, added_date, videoid, name, preview_image_location) VALUES (?, ?, ?, ?, ?);

-- name: VideosOfUser :many
SELECT videoid, added_date, name, preview_image_location
FROM user_videos WHERE userid = ? LIMIT 10;

-- name: AddLatestVideo :exec
INSERT INTO latest_videos (yyyymmdd, added_date, videoid, userid, name, preview_image_location) VALUES (?, ?, ?, ?, ?, ?);

-- name: LatestVideosOfDay :many
SELECT videoid, userid, name, added_date
FROM latest_videos WHERE yyyymmdd = ? LIMIT 20;

-- name: RateVideo :exec
INSERT INTO video_ratings_by_user (videoid, userid, rating) VALUES (?, ?, ?);

-- name: RatingsOfVideo :many
SELECT userid, rating FROM video_ratings_by_user WHERE videoid = ?;

-- name: CountPlayback :exec
UPDATE video_playback_stats SET views = views + 1 WHERE videoid = ?;

-- name: PlaybackStats :one
SELECT videoid, views FROM video_playback_stats WHERE videoid = ?;
