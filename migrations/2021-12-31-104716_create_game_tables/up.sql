CREATE TABLE events(
    id SERIAL PRIMARY KEY NOT NULL,
    name VARCHAR(255) NOT NULL,
    description VARCHAR(255) NOT NULL,
    compliance_factor FLOAT NOT NULL,
    infection_rate FLOAT NOT NULL,
    ideal_reproduction_number FLOAT NOT NULL
);

CREATE TABLE regions(
    id SERIAL PRIMARY KEY NOT NULL,
    susceptible FLOAT NOT NULL,
    exposed FLOAT NOT NULL,
    infected FLOAT NOT NULL,
    removed FLOAT NOT NULL,
    reproduction_number FLOAT NOT NULL,
    control_measure_levels INT,
    control_measure_isActive BOOLEAN NOT NULL
);

CREATE TABLE status(
    id SERIAL PRIMARY KEY,
    level_number INT NOT NULL,
    current_event INT NOT NULL,
    postponed INT NOT NULL,
    regions INT NOT NULL,
    CONSTRAINT fk_event FOREIGN KEY (current_event) REFERENCES events(id),
    CONSTRAINT fk_region FOREIGN KEY (regions) REFERENCES regions(id)
);

