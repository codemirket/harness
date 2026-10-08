-- Fictional legacy schema; immutable evaluation input.
CREATE TABLE customers (
    id INTEGER PRIMARY KEY,
    email TEXT NOT NULL,
    display_name TEXT NOT NULL,
    notes TEXT NOT NULL DEFAULT ''
);
CREATE INDEX customers_display_name ON customers(display_name);
CREATE TABLE email_audit (
    id INTEGER PRIMARY KEY,
    customer_id INTEGER NOT NULL,
    old_email TEXT NOT NULL,
    new_email TEXT NOT NULL
);
CREATE TRIGGER preserve_email_audit AFTER UPDATE OF email ON customers
BEGIN
    INSERT INTO email_audit(customer_id, old_email, new_email)
    VALUES (OLD.id, OLD.email, NEW.email);
END;
CREATE TABLE unrelated_settings (name TEXT PRIMARY KEY, value TEXT NOT NULL);
INSERT INTO unrelated_settings VALUES ('billing-mode', 'legacy');
INSERT INTO customers(id,email,display_name,notes) VALUES
 (10, '  ALICE@EXAMPLE.TEST ', 'Alice', 'retain spelling'),
 (20, 'bob@example.test', 'Bob', 'paid account'),
 (35, 'ALICE@example.test', 'Other Alice', 'duplicate key is allowed'),
 (40, ' CAROL@EXAMPLE.TEST', 'Carol', ''),
 (90, 'dave@example.test ', 'Dave', 'manual note'),
 (300, ' EVE@example.test ', 'Eve', 'unicode name: Éva'),
 (500, 'FRANK@example.test', 'Frank', 'late row');
