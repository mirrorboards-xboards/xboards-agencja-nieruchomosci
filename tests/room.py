"""A room of XEstates on SQLite in memory, for scenarios of its catalogs.

`Room(position)` reads the `meta` list of the position's `board.toml` the way XMeta does (own kinds in
its `meta/` and the own kinds of imported positions, by name or pattern; imports are not
transitive), applies the
migrations of the kinds of the room in the order the generator uses (a kind after the kinds whose
tables its migrations name), and runs operations by name the way the app does: the statements of
one operation in one transaction, a parameter left out bound as NULL.

Scenarios check the rules the database keeps: `refused` expects the database to refuse a write
and checks that the refusal names the rule.
"""

import fnmatch
import re
import sqlite3
import tomllib
from pathlib import Path

NAME_LINE = re.compile(r"^--\s*name:\s*(\S+)\s+:(many|one|exec)\s*$")
COMMENT_OR_STRING = re.compile(r"--[^\n]*|'(?:[^']|'')*'")


# What the guard triggers `<table>_bez_zastepowania` say. They fire before UNIQUE and CHECK, so a
# second row with an existing key is refused with this text, not with the name of the UNIQUE key.
REPLACEMENT_REFUSAL = "wiersza nie zastępuje się nowym wpisem"


class Parameters(dict):
    """Named parameters where a name left out binds NULL, as in the app's `bindStatement`."""

    def __missing__(self, name):
        return None


class Refused(AssertionError):
    pass


class Room:
    def __init__(self, position_directory):
        self.database = sqlite3.connect(":memory:")
        self.database.row_factory = sqlite3.Row

        kinds = _kinds_in_migration_order(_kinds_of_room(position_directory))
        for kind in kinds:
            for migration in sorted((kind / "migrations").glob("*.sql")):
                self.database.executescript(migration.read_text())

        # SQLite checks references only with this pragma on, and only after the schema exists.
        self.database.execute("PRAGMA foreign_keys = ON")

        self.operations = {}
        for kind in kinds:
            for name, statements in _operations_of(kind).items():
                if name in self.operations:
                    raise ValueError(f"operation {name} is in two kinds of the room")

                self.operations[name] = statements

    def run(self, operation, **parameters):
        """Runs an operation in one transaction; the rows of its last statement."""
        statements = self.operations.get(operation)
        if statements is None:
            raise KeyError(f"no operation {operation} in the room")

        rows = []
        with self.database:
            for statement in statements:
                rows = self.database.execute(statement, Parameters(parameters)).fetchall()

        return [dict(row) for row in rows]

    def execute(self, sql, **parameters):
        """A write no operation offers, e.g. by another kind or a careless writer."""
        with self.database:
            rows = self.database.execute(sql, Parameters(parameters)).fetchall()

        return [dict(row) for row in rows]

    def value(self, sql, **parameters):
        """The first column of the first row of a read."""
        row = self.database.execute(sql, Parameters(parameters)).fetchone()

        return None if row is None else row[0]

    def refused(self, rule, operation, **parameters):
        """Expects the database to refuse the operation with a message that contains `rule`."""
        self._expect_refusal(rule, lambda: self.run(operation, **parameters), operation)

    def refused_sql(self, rule, sql, **parameters):
        """Expects the database to refuse a write given as SQL."""
        self._expect_refusal(rule, lambda: self.execute(sql, **parameters), sql.split("\n")[0])

    def _expect_refusal(self, rule, write, what):
        try:
            write()
        except sqlite3.IntegrityError as error:
            if rule not in str(error):
                raise Refused(
                    f"{what}: refused for another reason than “{rule}”: {error}"
                ) from error

            return

        raise Refused(f"{what}: accepted, but the rule “{rule}” should refuse it")


def _kinds_of_room(position_directory):
    """The kind folders of the room: the imported ones in the order of the entries, then the own."""
    position = Path(position_directory).resolve()
    if not position.is_dir():
        raise ValueError(f"position {position_directory} does not exist")

    entries = _entries_of(position)

    kinds = []
    for folder, pattern in entries:
        if not _names_own_kinds(folder):
            kinds.extend(_imported_kinds(position, folder, pattern))

    kinds.extend(_own_kinds(position))

    names = [kind.name for kind in kinds]
    for name in set(names):
        if names.count(name) > 1 and len({kind for kind in kinds if kind.name == name}) > 1:
            raise ValueError(f"kind {name} comes from two positions")

    return list(dict.fromkeys(kinds))


def _entries_of(position):
    """The entries of `meta` in the position's `board.toml`: (kinds folder, name or pattern)."""
    file = position / "board.toml"
    if not file.is_file():
        raise ValueError(f"{position} has no board.toml, so it is no position")

    content = tomllib.loads(file.read_text())
    unknown_keys = set(content) - {"profile", "meta"}
    if unknown_keys:
        raise ValueError(f"{file}: unknown keys {sorted(unknown_keys)}")

    entries_of_meta = content.get("meta")
    is_list_of_strings = isinstance(entries_of_meta, list) and all(
        isinstance(item, str) for item in entries_of_meta
    )
    if not is_list_of_strings:
        raise ValueError(f"{file}: meta must be a list of strings")

    entries = []
    for entry in entries_of_meta:
        folder, _, pattern = entry.rpartition("/")
        if not folder:
            raise ValueError(f"{file}: entry {entry} names no kinds folder")

        entries.append((folder, pattern))

    return entries


def _names_own_kinds(folder):
    """Whether an entry's kinds folder is the `meta/` of its own position."""
    return folder.removeprefix("./") == "meta"


def _own_kinds(position):
    """The own kind folders of a position: the folders of its `meta/` its own entries name."""
    patterns = [pattern for folder, pattern in _entries_of(position) if _names_own_kinds(folder)]

    meta = position / "meta"
    folders = []
    if meta.is_dir():
        folders = sorted(folder for folder in meta.iterdir() if (folder / "migrations").is_dir())

    for pattern in patterns:
        if not any(fnmatch.fnmatchcase(folder.name, pattern) for folder in folders):
            raise ValueError(f"{position}: own entry {pattern} names no folder of meta/")

    for folder in folders:
        if not any(fnmatch.fnmatchcase(folder.name, pattern) for pattern in patterns):
            raise ValueError(f"{position}: folder meta/{folder.name} is not listed in board.toml")

    return folders


def _imported_kinds(position, folder, pattern):
    """The own kinds of another position named by an import entry (a name or a pattern with `*`)."""
    imported_position = (position / folder).resolve().parent
    if not imported_position.is_dir():
        raise ValueError(f"{position}: import {folder}/{pattern}: position does not exist")

    matching = [
        kind
        for kind in _own_kinds(imported_position)
        if fnmatch.fnmatchcase(kind.name, pattern)
    ]
    if not matching:
        raise ValueError(
            f"{position}: import {folder}/{pattern} matches no kind of {imported_position}"
        )

    return matching


def _kinds_in_migration_order(kinds):

    mentioned_kinds = {}
    for kind in kinds:
        migrations_sql = "\n".join(
            path.read_text() for path in sorted((kind / "migrations").glob("*.sql"))
        )
        words = set(re.findall(r"\w+", COMMENT_OR_STRING.sub(" ", migrations_sql)))

        mentioned_kinds[kind] = [
            other
            for other in kinds
            if other != kind and any(word.startswith(_table_prefix(other)) for word in words)
        ]

    ordered = []

    def visit(kind, path):
        if kind in ordered:
            return
        if kind in path:
            raise ValueError(f"kinds name each other in a circle: {[k.name for k in path]}")

        for required in mentioned_kinds[kind]:
            visit(required, [*path, kind])
        ordered.append(kind)

    for kind in kinds:
        visit(kind, [])

    return ordered


def _table_prefix(kind):
    return kind.name.replace("-", "_") + "_"


def _operations_of(kind):
    """`name → statements` of every operation file of a kind; the header comments left out."""
    operations = {}

    for path in sorted(kind.glob("*.sql")):
        name = None
        body = []

        for line in path.read_text().splitlines():
            name_line = NAME_LINE.match(line.strip())
            if name_line is None:
                body.append(line)
                continue

            if name is not None:
                operations[name] = _statements_of("\n".join(body))
            name = name_line.group(1)
            body = []

        if name is not None:
            operations[name] = _statements_of("\n".join(body))

    return operations


def _statements_of(sql):
    statements = []
    current = []

    for line in sql.splitlines():
        if not current and (line.strip() == "" or line.strip().startswith("--")):
            continue

        current.append(line)
        candidate = "\n".join(current)
        if sqlite3.complete_statement(candidate):
            statements.append(candidate)
            current = []

    return statements
