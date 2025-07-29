-- Analytics test data for aliases table
-- This file contains alias data designed to demonstrate analytics functionality
-- It creates multiple occurrences of common aliases across different domains

-- Insert seed data for analytics testing
INSERT INTO aliases (mail, destination, enabled) VALUES
-- postmaster appears 8 times (should be top analytics alias)
('postmaster@example.com', 'admin@example.com', 1),
('postmaster@test.com', 'admin@test.com', 1),
('postmaster@demo.com', 'admin@demo.com', 1),
('postmaster@sample.com', 'admin@sample.com', 1),
('postmaster@trial.com', 'admin@trial.com', 1),
('postmaster@dev.com', 'admin@dev.com', 1),
('postmaster@staging.com', 'admin@staging.com', 1),
('postmaster@prod.com', 'admin@prod.com', 1),

-- abuse appears 6 times (should be second analytics alias)
('abuse@example.com', 'admin@example.com', 1),
('abuse@test.com', 'admin@test.com', 1),
('abuse@demo.com', 'admin@demo.com', 1),
('abuse@sample.com', 'admin@sample.com', 1),
('abuse@trial.com', 'admin@trial.com', 1),
('abuse@dev.com', 'admin@dev.com', 1),

-- hostmaster appears 5 times (should be third analytics alias)
('hostmaster@example.com', 'admin@example.com', 1),
('hostmaster@test.com', 'admin@test.com', 1),
('hostmaster@demo.com', 'admin@demo.com', 1),
('hostmaster@sample.com', 'admin@sample.com', 1),
('hostmaster@trial.com', 'admin@trial.com', 1),

-- webmaster appears 4 times (should be fourth analytics alias)
('webmaster@example.com', 'admin@example.com', 1),
('webmaster@test.com', 'admin@test.com', 1),
('webmaster@demo.com', 'admin@demo.com', 1),
('webmaster@sample.com', 'admin@sample.com', 1),

-- info appears 3 times (should be fifth analytics alias)
('info@example.com', 'admin@example.com', 1),
('info@test.com', 'admin@test.com', 1),
('info@demo.com', 'admin@demo.com', 1),

-- support appears 3 times (should be sixth analytics alias)
('support@example.com', 'admin@example.com', 1),
('support@test.com', 'admin@test.com', 1),
('support@demo.com', 'admin@demo.com', 1),

-- sales appears 3 times (should be seventh analytics alias)
('sales@example.com', 'admin@example.com', 1),
('sales@test.com', 'admin@test.com', 1),
('sales@demo.com', 'admin@demo.com', 1),

-- marketing appears 3 times (should be eighth analytics alias)
('marketing@example.com', 'admin@example.com', 1),
('marketing@test.com', 'admin@test.com', 1),
('marketing@demo.com', 'admin@demo.com', 1),

-- contact appears 3 times (should be ninth analytics alias)
('contact@example.com', 'admin@example.com', 1),
('contact@test.com', 'admin@test.com', 1),
('contact@demo.com', 'admin@demo.com', 1),

-- help appears 3 times (should be tenth analytics alias)
('help@example.com', 'admin@example.com', 1),
('help@test.com', 'admin@test.com', 1),
('help@demo.com', 'admin@demo.com', 1),

-- Some unique aliases that shouldn't appear in analytics (less than 3 occurrences)
('unique1@example.com', 'admin@example.com', 1),
('unique2@test.com', 'admin@test.com', 1),
('unique3@demo.com', 'admin@demo.com', 1),

-- Catch-all aliases
('@example.com', 'admin@example.com', 1),
('@test.com', 'admin@test.com', 1),
('@demo.com', 'admin@demo.com', 1),
('@sample.com', 'admin@sample.com', 1),
('@trial.com', 'admin@trial.com', 1); 
