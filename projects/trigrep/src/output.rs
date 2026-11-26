//! Grep-style rendering of hits, with optional context lines.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use crate::search::Hit;

/// Render the hits of one file. Matches print as `path:line:text`, context as `path-line-text`,
/// and non-adjacent groups are separated by `--`.
pub fn render_file(path: &str, lines: &[String], matches: &[usize], context: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut last_printed = 0; // 1-based number of the last line emitted
    for &m in matches {
        let start = m.saturating_sub(context).max(1).max(last_printed + 1);
        if last_printed > 0 && start > last_printed + 1 {
            out.push("--".to_string());
        }
        let end = (m + context).min(lines.len());
        for n in start..=end {
            let sep = if matches.contains(&n) { ':' } else { '-' };
            out.push(format!("{path}{sep}{n}{sep}{}", lines[n - 1]));
        }
        last_printed = last_printed.max(end);
    }
    out
}

/// Render all hits grouped by file. With `context == 0` the original lines in `Hit` are used
/// and no file is re-read.
pub fn render(root: &Path, hits: &[Hit], context: usize) -> io::Result<Vec<String>> {
    let mut by_file: BTreeMap<&str, Vec<&Hit>> = BTreeMap::new();
    for h in hits {
        by_file.entry(h.path.as_str()).or_default().push(h);
    }
    let mut out = Vec::new();
    for (path, hs) in by_file {
        if context == 0 {
            out.extend(hs.iter().map(|h| format!("{path}:{}:{}", h.line_no, h.line)));
            continue;
        }
        let content = fs::read(root.join(path))?;
        let lines: Vec<String> = content
            .split(|&b| b == b'\n')
            .map(|l| String::from_utf8_lossy(l).trim_end_matches('\r').to_string())
            .collect();
        let nos: Vec<usize> = hs.iter().map(|h| h.line_no).collect();
        if !out.is_empty() {
            out.push("--".to_string());
        }
        out.extend(render_file(path, &lines, &nos, context));
    }
    Ok(out)
}

/// `path:count` for every file with at least one hit.
pub fn counts(hits: &[Hit]) -> Vec<String> {
    let mut by_file: BTreeMap<&str, usize> = BTreeMap::new();
    for h in hits {
        *by_file.entry(h.path.as_str()).or_default() += 1;
    }
    by_file.into_iter().map(|(p, c)| format!("{p}:{c}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("l{i}")).collect()
    }

    #[test]
    fn context_marks_matches_and_neighbours() {
        let out = render_file("f", &lines(5), &[3], 1);
        assert_eq!(out, vec!["f-2-l2", "f:3:l3", "f-4-l4"]);
    }

    #[test]
    fn overlapping_contexts_merge_without_duplicates() {
        let out = render_file("f", &lines(6), &[2, 4], 1);
        assert_eq!(out, vec!["f-1-l1", "f:2:l2", "f-3-l3", "f:4:l4", "f-5-l5"]);
    }

    #[test]
    fn distant_groups_are_separated() {
        let out = render_file("f", &lines(10), &[2, 9], 1);
        assert!(out.contains(&"--".to_string()));
        assert_eq!(out.len(), 7);
    }

    #[test]
    fn context_clamps_at_file_edges() {
        let out = render_file("f", &lines(2), &[1], 5);
        assert_eq!(out, vec!["f:1:l1", "f-2-l2"]);
    }

    #[test]
    fn counts_group_by_file() {
        let h = |p: &str, n| Hit { path: p.into(), line_no: n, line: String::new() };
        assert_eq!(counts(&[h("a", 1), h("a", 2), h("b", 1)]), vec!["a:2", "b:1"]);
    }
}
