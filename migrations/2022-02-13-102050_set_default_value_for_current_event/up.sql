-- Your SQL goes here
alter table status
alter current_event set default 0;

alter table regions
alter simulation_params set default '{"susceptible": 0.0, "exposed": 0.0, "infectious": 0.0, "removed": 0.0, "current_reproduction_number": 0.0, "ideal_reproduction_number": 0.0, "compliance_factor": 0.0, "recovery_rate": 0.0, "infection_rate": 0.0}';

