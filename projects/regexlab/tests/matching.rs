use regexlab::Regex;

fn find(p: &str, h: &str) -> Option<String> {
    let re = Regex::new(p).unwrap();
    re.find(h).map(|(a, b)| h[a..b].to_string())
}

#[test]
fn literals_and_dot() {
    assert_eq!(find("abc", "xxabcxx").as_deref(), Some("abc"));
    assert_eq!(find("a.c", "abc").as_deref(), Some("abc"));
    assert_eq!(find("a.c", "a\nc"), None);
    assert_eq!(find("abc", "ab"), None);
}

#[test]
fn quantifiers_greedy_and_lazy() {
    assert_eq!(find("a+", "caaat").as_deref(), Some("aaa"));
    assert_eq!(find("a+?", "caaat").as_deref(), Some("a"));
    assert_eq!(find("<.*>", "<a><b>").as_deref(), Some("<a><b>"));
    assert_eq!(find("<.*?>", "<a><b>").as_deref(), Some("<a>"));
    assert_eq!(find("a{2,3}", "aaaa").as_deref(), Some("aaa"));
    assert_eq!(find("a{2,3}?", "aaaa").as_deref(), Some("aa"));
    assert_eq!(find("ba{0}c", "bc").as_deref(), Some("bc"));
    assert_eq!(find("colou?r", "color").as_deref(), Some("color"));
}

#[test]
fn alternation_is_leftmost_first() {
    assert_eq!(find("a|ab", "ab").as_deref(), Some("a"));
    assert_eq!(find("ab|a", "ab").as_deref(), Some("ab"));
    assert_eq!(find("cat|dog", "hotdog").as_deref(), Some("dog"));
}

#[test]
fn anchors() {
    assert!(Regex::new("^abc$").unwrap().is_match("abc"));
    assert!(!Regex::new("^abc$").unwrap().is_match("xabc"));
    assert!(!Regex::new("^abc$").unwrap().is_match("abcx"));
    assert_eq!(find("$", "abc"), Some(String::new()));
}

#[test]
fn classes_and_shorthands() {
    assert_eq!(find(r"\d+", "ab123c").as_deref(), Some("123"));
    assert_eq!(find("[a-c]+", "xxabcabd").as_deref(), Some("abcab"));
    assert_eq!(find("[^a-c]+", "abxyc").as_deref(), Some("xy"));
    assert_eq!(find(r"\w+\s\w+", "hi  big world").as_deref(), Some("big world"));
}

#[test]
fn nested_empty_loops_terminate() {
    assert!(Regex::new("(a*)*b").unwrap().is_match("aaab"));
    assert!(!Regex::new("(a*)*b").unwrap().is_match("aaa"));
    assert!(Regex::new("(a|b*)*c").unwrap().is_match("abbac"));
}

#[test]
fn pathological_pattern_is_linear() {
    let n = 5000;
    let re = Regex::new(&format!("{}{}", "a?".repeat(25), "a".repeat(25))).unwrap();
    assert!(re.is_match(&"a".repeat(25)));
    let hay = "a".repeat(n);
    assert!(Regex::new("(a+)+b").unwrap().find(&hay).is_none());
}

#[test]
fn captures() {
    let re = Regex::new(r"(\d{4})-(\d\d)-(\d\d)").unwrap();
    let c = re.captures("on 2026-11-25.").unwrap();
    assert_eq!(c.get(0), Some("2026-11-25"));
    assert_eq!(c.get(1), Some("2026"));
    assert_eq!(c.get(3), Some("25"));

    let re = Regex::new("(a)|(b)").unwrap();
    let c = re.captures("b").unwrap();
    assert_eq!(c.get(1), None);
    assert_eq!(c.get(2), Some("b"));
}

#[test]
fn unicode_spans_are_byte_offsets() {
    let re = Regex::new("é+").unwrap();
    assert_eq!(re.find("caféé!"), Some((3, 7)));
}

#[test]
fn find_iter_handles_empty_matches() {
    let re = Regex::new(r"\d+").unwrap();
    assert_eq!(re.find_iter("a1b22c333"), vec![(1, 2), (3, 5), (6, 9)]);
    let re = Regex::new("x*").unwrap();
    assert_eq!(re.find_iter("ab").len(), 3);
}
