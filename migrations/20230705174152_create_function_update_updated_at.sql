-- vim: ft=pgsql

-- Trigger function that updates the `updated_at` attribute of a record to the
-- current time.
CREATE FUNCTION update_updated_at()
RETURNS trigger
LANGUAGE plpgsql
AS $$
	BEGIN
		NEW.updated_at = now();
		RETURN NEW;
	END;
$$;
