CREATE OR REPLACE FUNCTION export_tables_to_json()
RETURNS jsonb AS
$$
DECLARE
  rec record;
  table_json jsonb;
  result jsonb := '{}'::jsonb;
BEGIN
  FOR rec IN
    SELECT table_schema, table_name
    FROM information_schema.tables
    WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
      AND table_name NOT IN ('users', 'technical_details', 'metadata')
  LOOP
    EXECUTE format(
      'SELECT COALESCE(JSONB_AGG(ROW_TO_JSON(t)::jsonb), ''[]''::jsonb) FROM %I.%I t',
      rec.table_schema, rec.table_name
    )
    INTO table_json;
    
    result := result || jsonb_build_object(rec.table_name, table_json);
  END LOOP;
  
  RETURN result;
END;
$$ LANGUAGE plpgsql;