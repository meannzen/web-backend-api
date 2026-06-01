-- password: "secret123"
INSERT INTO users (id, email, password_hash, first_name, last_name, role) VALUES
    ('11111111-1111-1111-1111-111111111111', 'alice@example.com',
     '$argon2id$v=19$m=19456,t=2,p=1$dFdVZhCf7IUVXfu3UJbm6Q$5yUtdFaHs6bwC+AfybrfJzM6oYDTkAaHVVGDRJg+Yo4',
     'Alice', 'Smith', 'user'),
-- password: "secret456"
    ('22222222-2222-2222-2222-222222222222', 'bob@example.com',
     '$argon2id$v=19$m=19456,t=2,p=1$0f2E3DLuPUyWD7rVg5ElyQ$ZgOoQHPFzX2n3LGeI7M3y1WQ9ubdD4eYVvF7TFbrFrs',
     'Bob', 'Jones', 'user');
