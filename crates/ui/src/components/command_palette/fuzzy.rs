//! In-house fuzzy match/score, no external crate. See phase file's ADR
//! rationale: the corpus (a few dozen app commands) is far too small to
//! justify a real fuzzy-match library's complexity.

/// Scores `candidate` against `query`, case-insensitively. Returns `None` if
/// `query` is not a (possibly non-contiguous, in-order) subsequence of
/// `candidate` — i.e. no match at all. Higher scores rank better; an empty
/// `query` matches everything with score `0` (unfiltered default state).
pub fn score(query: &str, candidate: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }

    let query_lower = query.to_lowercase();
    let candidate_lower = candidate.to_lowercase();

    // A contiguous substring match always outranks a scattered subsequence
    // match, and ranks higher the closer it is to the start of the
    // candidate (prefix matches feel most relevant in a command list).
    if let Some(index) = candidate_lower.find(&query_lower) {
        let position_bonus = (100 - (index as i32).min(100)) * 10;
        let length_bonus = query_lower.chars().count() as i32;
        return Some(1_000 + position_bonus + length_bonus);
    }

    subsequence_score(&query_lower, &candidate_lower)
}

/// Scores an in-order, non-contiguous character subsequence match. Rewards
/// longer contiguous runs and matches that occur earlier in the candidate.
/// Returns `None` if `query`'s characters don't all appear, in order, in
/// `candidate`.
fn subsequence_score(query: &str, candidate: &str) -> Option<i32> {
    let query_chars: Vec<char> = query.chars().collect();
    if query_chars.is_empty() {
        return Some(0);
    }

    let mut query_ix = 0;
    let mut run_length = 0;
    let mut total_score = 0;
    let mut previous_match_ix: Option<usize> = None;

    for (candidate_ix, candidate_char) in candidate.chars().enumerate() {
        if query_ix >= query_chars.len() {
            break;
        }
        if candidate_char != query_chars[query_ix] {
            continue;
        }

        let is_contiguous = previous_match_ix.map(|ix| ix + 1) == Some(candidate_ix);
        run_length = if is_contiguous { run_length + 1 } else { 1 };
        total_score += run_length * 5 - (candidate_ix as i32) / 4;
        previous_match_ix = Some(candidate_ix);
        query_ix += 1;
    }

    if query_ix == query_chars.len() {
        Some(total_score)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_matches_everything() {
        assert_eq!(score("", "New Profile"), Some(0));
        assert_eq!(score("", ""), Some(0));
    }

    #[test]
    fn substring_match_is_case_insensitive() {
        assert!(score("prof", "New Profile").is_some());
        assert!(score("PROF", "new profile").is_some());
        assert!(score("Profile", "PROFILE").is_some());
    }

    #[test]
    fn no_subsequence_match_returns_none() {
        assert_eq!(score("xyz", "New Profile"), None);
        assert_eq!(score("zzz", ""), None);
    }

    #[test]
    fn scattered_subsequence_still_matches() {
        // "npf" -> N(ew) P(rofile) F(...)? not exactly, use a case that is a
        // genuine in-order (non-contiguous) subsequence.
        assert!(score("newprof", "New Profile").is_some());
    }

    #[test]
    fn contiguous_substring_ranks_above_scattered_subsequence() {
        let contiguous = score("prof", "New Profile").unwrap();
        let scattered = score("nwpf", "New Profile").unwrap();
        assert!(contiguous > scattered);
    }

    #[test]
    fn earlier_matches_rank_higher() {
        let early = score("new", "New Profile").unwrap();
        let late = score("file", "New Profile").unwrap();
        assert!(early > late);
    }

    #[test]
    fn out_of_order_characters_do_not_match() {
        // "c" appears before "a" in the query, but after it in "abc", so this
        // is not a valid in-order subsequence.
        assert_eq!(score("ca", "abc"), None);
    }
}
