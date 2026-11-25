import pytest

from diffsheet.loaders import LoadError, parse_table


def test_csv_basic():
    t = parse_table("id,name\n1,a\n2,b\n", "csv")
    assert t.columns == ["id", "name"]
    assert t.rows == [{"id": "1", "name": "a"}, {"id": "2", "name": "b"}]


def test_tsv():
    t = parse_table("id\tname\n1\ta\n", "tsv")
    assert t.rows == [{"id": "1", "name": "a"}]


def test_csv_duplicate_header():
    with pytest.raises(LoadError):
        parse_table("a,a\n1,2\n", "csv")


def test_json_array_unions_columns():
    t = parse_table('[{"id": 1}, {"id": 2, "x": "y"}]', "json")
    assert t.columns == ["id", "x"]
    assert t.rows[0] == {"id": 1, "x": None}


def test_json_object_of_objects():
    t = parse_table('{"a": {"v": 1}, "b": {"v": 2}}', "json")
    assert t.columns == ["_key", "v"]
    assert t.rows[1] == {"_key": "b", "v": 2}


@pytest.mark.parametrize("text", ["[1, 2]", '"s"', "{bad", '{"a": 1}'])
def test_json_rejects_bad_shapes(text):
    with pytest.raises(LoadError):
        parse_table(text, "json")


def test_unknown_format():
    with pytest.raises(LoadError):
        parse_table("", "xml")
