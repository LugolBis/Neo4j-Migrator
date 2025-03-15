CREATE OR REPLACE FUNCTION export_tables_metadata()
RETURNS JSON AS $$
DECLARE
    tables_metadata_json JSON;
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
    )
    
    SELECT tables_metadata 
    INTO tables_metadata_json
    FROM tables_info;
    
    RETURN tables_metadata_json;
END;
$$ LANGUAGE plpgsql;