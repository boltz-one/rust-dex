//! Hand-rolled subsequence fuzzy matching for Combobox and Command palette.
//!
//! No external crate — simple subsequence match with a basic relevance score
//! (consecutive-char and word-boundary bonuses) for ranking.

/// Returns `true` when every character of `query` appears in order within
/// `candidate` (case-insensitive). An empty query matches everything.
pub fn fuzzy_matches(query: &str, candidate: &str) -> bool {
    fuzzy_subsequence_score(query, candidate).is_some()
}

/// Scores a case-insensitive subsequence match. Higher is better.
/// Returns `None` when `query` is non-empty and not a subsequence of `candidate`.
pub fn fuzzy_subsequence_score(query: &str, candidate: &str) -> Option<usize> {
    let query = query.trim();
    if query.is_empty() {
        return Some(0);
    }

    let query_chars: Vec<char> = query.to_lowercase().chars().collect();
    let candidate_lower: Vec<char> = candidate.to_lowercase().chars().collect();

    let mut score = 0usize;
    let mut qi = 0usize;
    let mut last_match: Option<usize> = None;

    for (ci, &c) in candidate_lower.iter().enumerate() {
        if qi < query_chars.len() && c == query_chars[qi] {
            score += 1;
            if let Some(last) = last_match {
                if ci == last + 1 {
                    score += 2;
                }
            }
            if ci == 0 || candidate_lower.get(ci.saturating_sub(1)) == Some(&' ') {
                score += 3;
            }
            last_match = Some(ci);
            qi += 1;
        }
    }

    if qi == query_chars.len() {
        Some(score)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_matches_all() {
        assert!(fuzzy_matches("", "Anything"));
        assert_eq!(fuzzy_subsequence_score("", "Anything"), Some(0));
    }

    #[test]
    fn subsequence_match() {
        assert!(fuzzy_matches("fb", "Foo Bar"));
        assert!(!fuzzy_matches("bf", "Foo Bar"));
    }

    #[test]
    fn consecutive_bonus_ranks_higher() {
        let compact = fuzzy_subsequence_score("aa", "aaa").unwrap();
        let spread = fuzzy_subsequence_score("aa", "aba").unwrap();
        assert!(compact > spread);
    }
}
