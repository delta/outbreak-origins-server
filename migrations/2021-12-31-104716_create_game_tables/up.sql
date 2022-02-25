CREATE TABLE regions(
    id SERIAL PRIMARY KEY,
    region_id INT NOT NULL,
    simulation_params jsonb DEFAULT '{}'::jsonb NOT NULL,
    active_control_measures jsonb DEFAULT '{}'::jsonb NOT NULL
);

CREATE TABLE status(
    id SERIAL PRIMARY KEY,
    current_event INT NOT NULL,
    postponed INT DEFAULT 0 NOT NULL
);

ALTER TABLE users
ADD COLUMN status INT REFERENCES status(id);

-- For many to one relation between regions and status
CREATE TABLE regions_status (
    id SERIAL PRIMARY KEY,
    status_id INT REFERENCES status(id) NOT NULL,
    region_id INT REFERENCES regions(id) NOT NULL
);
