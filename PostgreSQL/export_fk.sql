CREATE OR REPLACE FUNCTION export_fk_relationships() RETURNS JSONB AS $$
DECLARE
    result jsonb := '{}'::jsonb;
    fk_record RECORD;
    source_pk_cols TEXT;
    referenced_pk_cols TEXT;
    join_conditions TEXT;
    query TEXT;
    pairs JSONB;
    key_name TEXT;
BEGIN
    FOR fk_record IN
        SELECT
            tc.table_name as source_table,
            ccu.table_name as referenced_table,
            kcu.column_name as fk_column,
            ccu.column_name as referenced_column,
            tc.constraint_name
        FROM 
            information_schema.table_constraints AS tc 
            JOIN information_schema.key_column_usage AS kcu
              ON tc.constraint_name = kcu.constraint_name
            JOIN information_schema.constraint_column_usage AS ccu
              ON tc.constraint_name = ccu.constraint_name
        WHERE 
            tc.constraint_type = 'FOREIGN KEY'
    LOOP
        EXECUTE format(
            'SELECT string_agg(column_name, '','') 
             FROM information_schema.key_column_usage 
             WHERE table_name = %L 
             AND constraint_name IN (
                 SELECT constraint_name 
                 FROM information_schema.table_constraints 
                 WHERE table_name = %L 
                 AND constraint_type = ''PRIMARY KEY''
             )',
            fk_record.source_table,
            fk_record.source_table
        ) INTO source_pk_cols;

        EXECUTE format(
            'SELECT string_agg(column_name, '','') 
             FROM information_schema.key_column_usage 
             WHERE table_name = %L 
             AND constraint_name IN (
                 SELECT constraint_name 
                 FROM information_schema.table_constraints 
                 WHERE table_name = %L 
                 AND constraint_type = ''PRIMARY KEY''
             )',
            fk_record.referenced_table,
            fk_record.referenced_table
        ) INTO referenced_pk_cols;

        join_conditions := format(
            '%I.%I = %I.%I',
            fk_record.source_table,
            fk_record.fk_column,
            fk_record.referenced_table,
            fk_record.referenced_column
        );

        query := format(
            'SELECT jsonb_agg(jsonb_build_array(source_pk, referenced_pk)) 
             FROM (
                 SELECT 
                     (%I.%I) as source_pk,
                     (%I.%I) as referenced_pk 
                 FROM 
                     %I 
                     JOIN %I ON %s
             ) sub',
            fk_record.source_table, source_pk_cols,
            fk_record.referenced_table, referenced_pk_cols,
            fk_record.source_table,
            fk_record.referenced_table,
            join_conditions
        );

        EXECUTE query INTO pairs;
        
        key_name := format('%s_REF_%s',UPPER(fk_record.source_table),UPPER(fk_record.referenced_table));
        
        result := result || jsonb_build_object(key_name, pairs)::jsonb;
    END LOOP;

    RETURN result;
END;
$$ LANGUAGE plpgsql;