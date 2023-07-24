-- Add up migration script here
INSERT INTO users 
(id, username, password_hash) VALUES 
('391afe4c-6c47-4719-bc1f-3aca3600b8db', 'boris', '$argon2id$v=19$m=15000,t=2,p=1$+5Q2MhPnD/U86vgTlnB5wg$HlrNDJfFz1eeQuBV1D2xrqQpdu31mVazQetxA+eW59M');
