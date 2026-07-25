//! Fuzzy matching for service names (exact, substring, Levenshtein).

use crate::models::{FuzzyMatch, MatchType, Service};

pub fn levenshtein_distance(left: &str, right: &str) -> usize {
    let left_chars: Vec<char> = left.chars().collect();
    let right_chars: Vec<char> = right.chars().collect();
    let left_len = left_chars.len();
    let right_len = right_chars.len();
    if left_len == 0 {
        return right_len;
    }
    if right_len == 0 {
        return left_len;
    }

    let mut previous = (0..=right_len).collect::<Vec<_>>();
    let mut current = vec![0; right_len + 1];

    for (index_left, left_char) in left_chars.iter().enumerate() {
        current[0] = index_left + 1;
        for (index_right, right_char) in right_chars.iter().enumerate() {
            let cost = usize::from(left_char != right_char);
            current[index_right + 1] = (previous[index_right + 1] + 1)
                .min(current[index_right] + 1)
                .min(previous[index_right] + cost);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[right_len]
}

pub fn similarity_ratio(left: &str, right: &str) -> f64 {
    if left == right {
        return 1.0;
    }
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    let max_len = left.chars().count().max(right.chars().count());
    let distance = levenshtein_distance(left, right);
    1.0 - (distance as f64 / max_len as f64)
}

pub fn find_services_fuzzy(services: &[Service], query: &str, threshold: f64) -> Vec<FuzzyMatch> {
    let query_lower = query.trim().to_lowercase();
    if query_lower.is_empty() {
        return Vec::new();
    }

    let mut results = Vec::new();
    for service in services {
        let name_lower = service.name.to_lowercase();
        if name_lower == query_lower {
            results.push(FuzzyMatch {
                service: service.clone(),
                score: 1.0,
                match_type: MatchType::Exact,
            });
            continue;
        }
        if name_lower.contains(&query_lower) || query_lower.contains(&name_lower) {
            let match_length = name_lower.len().min(query_lower.len());
            let max_length = name_lower.len().max(query_lower.len());
            let score = match_length as f64 / max_length as f64;
            results.push(FuzzyMatch {
                service: service.clone(),
                score,
                match_type: MatchType::Substring,
            });
            continue;
        }
        let similarity = similarity_ratio(&name_lower, &query_lower);
        if similarity >= threshold {
            results.push(FuzzyMatch {
                service: service.clone(),
                score: similarity,
                match_type: MatchType::Fuzzy,
            });
        }
    }

    results.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.service.name.cmp(&right.service.name))
    });
    results
}

pub fn dedupe_by_name(matches: Vec<FuzzyMatch>) -> Vec<FuzzyMatch> {
    let mut seen = std::collections::HashSet::new();
    matches
        .into_iter()
        .filter(|item| seen.insert(item.service.name.to_lowercase()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match_scores_one() {
        let services = vec![Service {
            name: "GitHub".into(),
            status_url: Some("https://www.githubstatus.com".into()),
            website_url: None,
            security_url: None,
            support_url: None,
            aux_urls: vec![],
        }];
        let results = find_services_fuzzy(&services, "github", 0.5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].match_type, MatchType::Exact);
        assert!((results[0].score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn levenshtein_basic() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("", "abc"), 3);
    }
}
