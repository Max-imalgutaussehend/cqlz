-- name: CreateCustomer :exec
INSERT INTO customers (customer_id, name, email, vip, birthday, created_at, tags, phones)
VALUES (?, ?, ?, ?, ?, toTimestamp(now()), {'new'}, ['none']);

-- name: CreateCustomerFromJson :exec
INSERT INTO customers JSON ?;

-- name: CreateCustomerIfAbsent :lwt
INSERT INTO customers (customer_id, name, email) VALUES (?, ?, ?) IF NOT EXISTS;

-- name: GetCustomer :one
SELECT customer_id, name, email, vip, birthday, created_at, primary_address, addresses, tags, phones
FROM customers WHERE customer_id = ?;

-- name: GetCustomerJson :one
SELECT JSON name, email FROM customers WHERE customer_id = ?;

-- name: CustomerMeta :one
SELECT writetime(email) AS email_written_at, ttl(name) AS name_ttl, token(customer_id) AS tok,
       toUnixTimestamp(created_at) AS created_unix, primary_address.city AS city,
       cast(vip AS text) AS vip_text
FROM customers WHERE customer_id = ?;

-- allow: CQL201
-- name: CountCustomers :one
SELECT count(*) AS total FROM customers;

-- name: SetPrimaryAddress :exec
UPDATE customers SET primary_address = ? WHERE customer_id = ?;

-- name: AddTag :exec
UPDATE customers SET tags = tags + ? WHERE customer_id = ?;

-- name: RemoveTag :exec
UPDATE customers SET tags = tags - ? WHERE customer_id = ?;

-- name: PrependPhone :exec
UPDATE customers SET phones = ? + phones WHERE customer_id = ?;

-- name: SetAddress :exec
UPDATE customers SET addresses[?] = ? WHERE customer_id = ?;

-- name: ClearAddress :exec
DELETE addresses[?] FROM customers WHERE customer_id = ?;

-- name: ClearCollections :exec
DELETE tags, phones FROM customers WHERE customer_id = ?;

-- name: RenameCustomer :exec
UPDATE customers USING TTL ? SET name = ? WHERE customer_id = ?;

-- name: CustomersByIds :many
SELECT customer_id, name FROM customers WHERE customer_id IN ?;

-- allow: CQL201, CQL202
-- name: CustomersByTag :many
SELECT customer_id, name FROM customers WHERE tags CONTAINS ? ALLOW FILTERING;

-- name: ScanByToken :many
SELECT customer_id FROM customers WHERE token(customer_id) > token(?) LIMIT ?;

-- name: DeleteCustomerIfExists :lwt
DELETE FROM customers WHERE customer_id = ? IF EXISTS;
-- name: CountView :exec
UPDATE product_views SET views = views + ? WHERE product_id = ?;

-- name: ViewCount :one
SELECT views FROM product_views WHERE product_id = ?;

-- name: AddReading :exec
INSERT INTO sensor_readings (sensor_id, day, ts, value) VALUES (?, ?, ?, ?);

-- name: ReadingsOfDay :many
SELECT ts, value FROM sensor_readings
WHERE sensor_id = ? AND day = ? AND ts >= ? AND ts < ?;

-- name: ReadingStats :one
SELECT count(*) AS n, min(value) AS lo, max(value) AS hi, avg(value) AS mean
FROM sensor_readings WHERE sensor_id = ? AND day = ?;

-- name: ReserveStock :lwt
UPDATE inventory SET reserved = ? WHERE sku = ? IF stock >= ?;

-- name: CreateSession :exec
INSERT INTO sessions (session_token, customer_id) VALUES (?, ?) USING TTL ?;

-- name: SessionCustomer :one
SELECT customer_id FROM sessions WHERE session_token = ?;
-- name: PlaceOrder :exec
INSERT INTO orders_by_customer (customer_id, order_time, order_id, status, total)
VALUES (?, now(), uuid(), 'new', ?);

-- name: OrdersSince :many
SELECT order_time, order_id, status, total, toTimestamp(order_time) AS placed_at
FROM orders_by_customer
WHERE customer_id = ? AND order_time > minTimeuuid(?) AND order_time < maxTimeuuid(?)
ORDER BY order_time DESC
LIMIT ?;

-- name: OrdersPage :many
SELECT order_time, order_id, status FROM orders_by_customer
WHERE customer_id = ? AND (order_time, order_id) < (?, ?)
LIMIT ?;

-- name: CountOrders :one
SELECT count(*) AS n FROM orders_by_customer WHERE customer_id = ?;

-- name: Totals :one
SELECT min(total) AS lowest, max(total) AS highest, sum(total) AS sum_total, avg(total) AS avg_total
FROM orders_by_customer WHERE customer_id = ?;

-- allow: CQL201
-- name: CustomersWithOrders :many
SELECT DISTINCT customer_id FROM orders_by_customer;

-- allow: CQL201
-- name: LatestOrderPerCustomer :many
SELECT customer_id, order_time, status FROM orders_by_customer PER PARTITION LIMIT 1;

-- allow: CQL201
-- name: OrdersPerCustomer :many
SELECT customer_id, count(*) AS n FROM orders_by_customer GROUP BY customer_id;

-- name: ShipOrder :lwt
UPDATE orders_by_customer SET status = 'shipped'
WHERE customer_id = ? AND order_time = ? AND order_id = ? IF status = ?;
