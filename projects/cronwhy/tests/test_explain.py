import pytest

from cronwhy import parse
from cronwhy.explain import explain


@pytest.mark.parametrize("expr,text", [
    ("* * * * *", "Every minute"),
    ("*/5 * * * *", "Every 5 minutes"),
    ("0 * * * *", "Every hour, on the hour"),
    ("30 9 * * *", "At 09:30"),
    ("0 9 * * mon-fri", "At 09:00, on Monday through Friday"),
    ("0 0 1 * *", "At 00:00, on the 1st"),
    ("0 0 1,15 * *", "At 00:00, on the 1st and 15th"),
    ("0 8,17 * * *", "At 08:00 and 17:00"),
    ("15 3 * jan,jul *", "At 03:15, every day, in January and July"),
    ("0 0 13 * fri", "At 00:00, on the 13th or on Friday"),
    ("0 0 * * 0,6", "At 00:00, on Sunday and Saturday"),
    ("@yearly", "At 00:00, on the 1st, in January"),
    ("0 */6 * * *", "At minute 0 every 6 hours"),
])
def test_explain(expr, text):
    assert explain(parse(expr)) == text
