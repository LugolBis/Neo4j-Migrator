CREATE OR REPLACE FUNCTION get_database_metadata()
RETURNS JSON AS $$
DECLARE
    tables_metadata_json JSON;
    triggers_metadata_json JSON;
    procedures_metadata_json JSON;
    functions_metadata_json JSON;
BEGIN
    -- Récupérer les métadonnées des tables, triggers, procédures et fonctions en une seule requête
    WITH tables_info AS (
        SELECT json_agg(
            json_build_object(
                'table_name', c.table_name,
                'columns', (
                    SELECT json_agg(
                        json_build_object(
                            'column_name', cols.column_name,
                            'data_type', cols.data_type,
                            'is_nullable', cols.is_nullable,
                            'column_default', cols.column_default,
                            'primary_key', (
                                SELECT bool_or(kcu.column_name IS NOT NULL)
                                FROM information_schema.table_constraints tc
                                JOIN information_schema.key_column_usage kcu 
                                  ON tc.constraint_name = kcu.constraint_name 
                                  AND tc.table_schema = kcu.table_schema
                                WHERE tc.table_name = c.table_name 
                                  AND tc.table_schema = 'public'
                                  AND tc.constraint_type = 'PRIMARY KEY'
                                  AND kcu.column_name = cols.column_name
                            ),
                            'foreign_key', (
                                SELECT json_agg(
                                    json_build_object(
                                        'constraint_name', con.conname,
                                        'referenced_table', fr.relname,
                                        'referenced_column', fratt.attname
                                    )
                                )
                                FROM pg_constraint con
                                JOIN pg_class cl ON con.conrelid = cl.oid
                                JOIN pg_attribute att ON att.attrelid = cl.oid AND att.attnum = ANY(con.conkey)
                                JOIN pg_class fr ON con.confrelid = fr.oid
                                JOIN pg_attribute fratt ON fratt.attrelid = fr.oid AND fratt.attnum = ANY(con.confkey)
                                WHERE cl.relname = c.table_name AND con.contype = 'f' AND att.attname = cols.column_name
                            )
                        )
                    )
                    FROM information_schema.columns cols
                    WHERE cols.table_name = c.table_name AND cols.table_schema = 'public'
                )
            )
            ORDER BY pgc.oid
        ) AS tables_metadata
        FROM information_schema.tables c
        JOIN pg_class pgc ON c.table_name = pgc.relname
        WHERE c.table_schema = 'public'
    ),
    
    triggers_info AS (
        SELECT json_agg(
            json_build_object(
                'trigger_name', tg.tgname,
                'event', pg_get_triggerdef(tg.oid),
                'table_name', rel.relname
            )
        ) AS triggers_metadata
        FROM pg_trigger tg
        JOIN pg_class rel ON tg.tgrelid = rel.oid
        WHERE NOT tg.tgisinternal
    ),
    
    procedures_info AS (
        SELECT json_agg(
            json_build_object(
                'procedure_name', p.proname,
                'return_type', t.typname,
                'arguments', pg_catalog.pg_get_function_result(p.oid),
                'definition', pg_catalog.pg_get_functiondef(p.oid)
            )
        ) AS procedures_metadata
        FROM pg_proc p
        JOIN pg_namespace n ON p.pronamespace = n.oid
        JOIN pg_type t ON p.prorettype = t.oid
        WHERE n.nspname = 'public' AND p.prokind = 'p'
    ),
    
    functions_info AS (
        SELECT json_agg(
            json_build_object(
                'function_name', p.proname,
                'return_type', t.typname,
                'arguments', pg_catalog.pg_get_function_result(p.oid),
                'definition', pg_catalog.pg_get_functiondef(p.oid)
            )
        ) AS functions_metadata
        FROM pg_proc p
        JOIN pg_namespace n ON p.pronamespace = n.oid
        JOIN pg_type t ON p.prorettype = t.oid
        WHERE n.nspname = 'public' AND p.prokind = 'f'
    )
    
    SELECT tables_metadata, triggers_metadata, procedures_metadata, functions_metadata 
    INTO tables_metadata_json, triggers_metadata_json, procedures_metadata_json, functions_metadata_json
    FROM tables_info, triggers_info, procedures_info, functions_info;
    
    RETURN json_build_object(
        'tables', tables_metadata_json,
        'triggers', triggers_metadata_json,
        'procedures', procedures_metadata_json,
        'functions', functions_metadata_json
    );
END;
$$ LANGUAGE plpgsql;