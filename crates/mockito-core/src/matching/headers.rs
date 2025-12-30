//! Headers intersection check (case-insensitive).

use std::collections::HashMap;

fn normalize_headers(headers: Option<&HashMap<String, String>>) -> HashMap<String, String> {
    headers
        .map(|h| {
            h.iter()
                .map(|(k, v)| (k.to_lowercase(), v.clone()))
                .collect()
        })
        .unwrap_or_default()
}

pub fn headers_intersects(
    target: Option<&HashMap<String, String>>,
    subset: Option<&HashMap<String, String>>,
) -> bool {
    let subset = match subset {
        None => return true,
        Some(s) if s.is_empty() => return true,
        Some(s) => s,
    };

    let target = match target {
        None => return false,
        Some(t) => t,
    };

    let target = normalize_headers(Some(target));
    let subset = normalize_headers(Some(subset));

    subset.iter().all(|(k, v)| target.get(k) == Some(v))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn h(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).into(), (*v).into()))
            .collect()
    }

    #[rstest]
    #[case(None, None, true)]
    #[case(Some(&h(&[])), Some(&h(&[])), true)]
    #[case(Some(&h(&[("Content-Type", "application/json")])), None, true)]
    #[case(Some(&h(&[("Content-Type", "application/json")])), Some(&h(&[])), true)]
    #[case(Some(&h(&[("Content-Type", "application/json"), ("Auth", "Bearer x")])), Some(&h(&[("content-type", "application/json")])), true)]
    #[case(Some(&h(&[("Content-Type", "application/json")])), Some(&h(&[("Content-Type", "text/plain")])), false)]
    #[case(None, Some(&h(&[("Content-Type", "application/json")])), false)]
    #[case(Some(&h(&[("Accept", "text/html")])), Some(&h(&[("Content-Type", "application/json")])), false)]
    fn test_headers_intersects(
        #[case] target: Option<&HashMap<String, String>>,
        #[case] subset: Option<&HashMap<String, String>>,
        #[case] expected: bool,
    ) {
        assert_eq!(headers_intersects(target, subset), expected);
    }
}
