"""One check for every catalog: no table lets a row be replaced by INSERT OR REPLACE or an upsert.

SQLite resolves both by deleting or updating the existing row without the BEFORE UPDATE and
BEFORE DELETE triggers of the table (a room does not enable `recursive_triggers`). Every table has
a trigger `<table>_bez_zastepowania` that refuses an INSERT whose primary key or UNIQUE key already
belongs to a row. This module tries to get around it on every table of a room.
"""

import re

from room import REPLACEMENT_REFUSAL

PLACEHOLDER_OF_TYPE = {
    "INTEGER": "1",
    "TEXT": "'x'",
    "REAL": "1.0",
    "BLOB": "x'00'",
    "ANY": "'x'",
}
OTHER_PLACEHOLDER_OF_TYPE = {
    "INTEGER": "2",
    "TEXT": "'y'",
    "REAL": "2.0",
    "BLOB": "x'01'",
    "ANY": "'y'",
}


def refuse_replacing_every_row(room):
    """Fills every table of the room with one row and tries to replace that row in three ways.

    A row is replaced by a copy with the same primary key (INSERT OR REPLACE, upsert) or, for each
    UNIQUE key, by a copy that keeps only that key. The rows are synthetic: the other triggers and
    the CHECK constraints are switched off, so that only the guard is under test.
    """
    database = room.database
    database.execute("PRAGMA foreign_keys = OFF")
    database.execute("PRAGMA ignore_check_constraints = ON")

    other_triggers = database.execute(
        "SELECT name FROM sqlite_schema WHERE type = 'trigger' AND name NOT LIKE '%_bez_zastepowania'"
    ).fetchall()
    for trigger in other_triggers:
        database.execute(f"DROP TRIGGER {trigger['name']}")

    tables = [
        row["name"]
        for row in database.execute(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name NOT LIKE 'sqlite\\_%' ESCAPE '\\'"
        )
    ]
    assert tables, "the room has no table"

    for table in tables:
        guard = f"{table}_bez_zastepowania"
        guard_exists = room.value(
            "SELECT count(*) FROM sqlite_schema WHERE type = 'trigger' AND name = :name",
            name=guard,
        )
        assert guard_exists == 1, f"{table} has no trigger {guard}"

        _refuse_replacing_row_of(room, table)


def _refuse_replacing_row_of(room, table):
    columns = room.execute(f"PRAGMA table_info({table})")
    column_names = [column["name"] for column in columns]
    column_list = ", ".join(column_names)
    primary_key = next(column for column in columns if column["pk"] == 1)

    # Columns that a partial UNIQUE index requires to be NULL stay NULL, or the row is outside it.
    null_columns = _columns_that_partial_indexes_require_null(room, table)
    placeholders = {
        column["name"]: "NULL"
        if column["name"] in null_columns
        else PLACEHOLDER_OF_TYPE[column["type"]]
        for column in columns
    }
    room.execute(f"INSERT INTO {table} ({column_list}) VALUES ({', '.join(placeholders.values())})")

    copy = f"INTO {table} ({column_list}) SELECT {column_list} FROM {table} WHERE rowid = 1"
    room.refused_sql(REPLACEMENT_REFUSAL, f"INSERT OR REPLACE {copy}")
    room.refused_sql(
        REPLACEMENT_REFUSAL,
        f"INSERT {copy} ON CONFLICT ({primary_key['name']}) DO UPDATE SET {primary_key['name']} = excluded.{primary_key['name']}",
    )

    # A copy with another primary key collides only on the UNIQUE key it keeps from the first row.
    unique_keys = _unique_keys_of(room, table)
    for key in unique_keys:
        other_keys_columns = {column for other in unique_keys if other != key for column in other}
        changed_columns = ({primary_key["name"]} | other_keys_columns) - set(key)
        selected = ", ".join(
            OTHER_PLACEHOLDER_OF_TYPE[_type_of(columns, name)] if name in changed_columns else name
            for name in column_names
        )
        room.refused_sql(
            REPLACEMENT_REFUSAL,
            f"INSERT OR REPLACE INTO {table} ({column_list}) SELECT {selected} FROM {table} WHERE rowid = 1",
        )


def _type_of(columns, name):
    return next(column["type"] for column in columns if column["name"] == name)


def _unique_keys_of(room, table):
    """The columns of every UNIQUE constraint and UNIQUE index of the table, except the primary key."""
    keys = []

    for index in room.execute(f"PRAGMA index_list({table})"):
        if index["unique"] != 1 or index["origin"] == "pk":
            continue

        key = [column["name"] for column in room.execute(f"PRAGMA index_info({index['name']})")]
        if None in key:
            # An index on an expression, such as coalesce(testament_id, ''): take its column.
            sql = room.value("SELECT sql FROM sqlite_schema WHERE name = :name", name=index["name"])
            key = [name for name in key if name is not None] + re.findall(r"coalesce\((\w+),", sql)
        keys.append(key)

    return keys


def _columns_that_partial_indexes_require_null(room, table):
    columns = set()

    for index in room.execute(f"PRAGMA index_list({table})"):
        if index["partial"] != 1:
            continue

        sql = room.value("SELECT sql FROM sqlite_schema WHERE name = :name", name=index["name"])
        columns.update(re.findall(r"WHERE (\w+) IS NULL", sql))

    return columns
