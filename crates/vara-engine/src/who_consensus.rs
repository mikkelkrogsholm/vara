//! Cross-window agreement for known-count Who. No embeddings leave the SDK workers.
use crate::{Cancellation, Result};
use std::collections::{BTreeMap, BTreeSet};
use vara_core::{Diarization, SpeakerTurn};

fn bad() -> String {
    "Who kunne ikke afstemme talerne mellem kontrolkørslerne. Tidligere resultat er bevaret.".into()
}
const UNKNOWN: &str = "who-uncertain";

// Intervals between all endpoints, with complete sets (including uncertainty).
fn intervals(
    a: &[SpeakerTurn],
    b: &[SpeakerTurn],
) -> Vec<(f64, f64, BTreeSet<String>, BTreeSet<String>)> {
    let mut events = Vec::with_capacity((a.len() + b.len()) * 2);
    for (side, turns) in [a, b].into_iter().enumerate() {
        for t in turns {
            events.push((t.start, side, t.speaker.clone(), 1_i32));
            events.push((t.end, side, t.speaker.clone(), -1_i32));
        }
    }
    events.sort_by(|a, b| a.0.total_cmp(&b.0));
    // Snap tiny numerical jitter to a primary endpoint when available.
    // Distinct primary endpoints are never merged, preserving primary activity.
    let mut i = 0;
    while i < events.len() {
        let first = events[i].0;
        let mut primary = None;
        let mut end = i;
        while end < events.len() && events[end].0 - first <= 1e-8 {
            if events[end].1 == 0 {
                if primary.is_some_and(|t| t != events[end].0) {
                    break;
                }
                primary = Some(events[end].0);
            }
            end += 1;
        }
        let time = primary.unwrap_or(first);
        for event in &mut events[i..end] {
            event.0 = time;
        }
        i = end;
    }
    let mut active = [BTreeMap::<String, i32>::new(), BTreeMap::new()];
    let mut out = Vec::new();
    let mut i = 0;
    while i < events.len() {
        let time = events[i].0;
        while i < events.len() && events[i].0 == time {
            let (_, side, ref speaker, delta) = events[i];
            let n = active[side].entry(speaker.clone()).or_default();
            *n += delta;
            if *n == 0 {
                active[side].remove(speaker);
            }
            i += 1;
        }
        if i < events.len() && events[i].0 > time {
            out.push((
                time,
                events[i].0,
                active[0].keys().cloned().collect(),
                active[1].keys().cloned().collect(),
            ));
        }
    }
    out
}

// Exact subset dynamic programming, O(K * 2^K), including the runner-up score
// so equal global optima are rejected. Every selected edge must be positive.
fn assignment(weights: &[Vec<f64>]) -> Result<Vec<usize>> {
    let n = weights.len();
    if n == 0 || n > 16 || weights.iter().any(|r| r.len() != n) {
        return Err(bad());
    }
    let size = 1usize << n;
    let mut best = vec![f64::NEG_INFINITY; size];
    let mut second = best.clone();
    let mut choice = vec![usize::MAX; size];
    best[0] = 0.;
    for mask in 1..size {
        let row = mask.count_ones() as usize - 1;
        for col in 0..n {
            if mask & (1 << col) == 0 || weights[row][col] <= 0. {
                continue;
            }
            let prev = mask ^ (1 << col);
            for (rank, base) in [best[prev], second[prev]].into_iter().enumerate() {
                let score = base + weights[row][col];
                if score > best[mask] {
                    second[mask] = best[mask];
                    best[mask] = score;
                    choice[mask] = if rank == 0 { col } else { usize::MAX };
                } else if score > second[mask] {
                    second[mask] = score;
                }
            }
        }
    }
    let mut mask = size - 1;
    if !best[mask].is_finite() || best[mask] - second[mask] <= 1e-7 {
        return Err(bad());
    }
    let mut out = vec![0; n];
    while mask != 0 {
        let row = mask.count_ones() as usize - 1;
        let col = choice[mask];
        if col == usize::MAX {
            return Err(bad());
        }
        out[row] = col;
        mask ^= 1 << col;
    }
    Ok(out)
}

pub(super) fn combine(
    mut primary: Diarization,
    secondary: Diarization,
    duration: f64,
    cancel: &Cancellation,
) -> Result<Diarization> {
    let count = primary.known_speakers.as_ref().ok_or_else(bad)?.count as usize;
    let secondary_turns: Vec<_> = secondary
        .turns
        .into_iter()
        .filter_map(|t| {
            let start = (t.start - 60.).max(0.);
            let end = (t.end - 60.).min(duration);
            (end > start).then_some(SpeakerTurn {
                speaker: t.speaker,
                start,
                end,
            })
        })
        .collect();
    let ids = |turns: &[SpeakerTurn]| -> Vec<String> {
        turns
            .iter()
            .filter(|t| t.speaker != UNKNOWN)
            .map(|t| t.speaker.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    };
    let left = ids(&primary.turns);
    let right = ids(&secondary_turns);
    if left.len() != count || right.len() != count {
        return Err(bad());
    }
    let spans = intervals(&primary.turns, &secondary_turns);
    let mut weights = vec![vec![0.; count]; count];
    for (start, end, a, b) in &spans {
        if a.len() == 1 && b.len() == 1 && !a.contains(UNKNOWN) && !b.contains(UNKNOWN) {
            let i = left.binary_search(a.first().unwrap()).map_err(|_| bad())?;
            let j = right.binary_search(b.first().unwrap()).map_err(|_| bad())?;
            weights[i][j] += end - start;
        }
    }
    let mapping = assignment(&weights)?;
    let map: BTreeMap<_, _> = mapping
        .iter()
        .enumerate()
        .map(|(i, &j)| (right[j].clone(), left[i].clone()))
        .collect();
    let mut by_speaker: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    for (index, (start, end, a, b)) in spans.into_iter().enumerate() {
        if index % 1000 == 0 {
            cancel.check()?;
        }
        if a.is_empty() {
            continue;
        }
        let mapped: BTreeSet<_> = b
            .iter()
            .map(|s| {
                if s == UNKNOWN {
                    s.clone()
                } else {
                    map.get(s).cloned().unwrap_or_else(|| UNKNOWN.into())
                }
            })
            .collect();
        let speakers = if !a.contains(UNKNOWN) && !mapped.contains(UNKNOWN) && a == mapped {
            a
        } else {
            BTreeSet::from([UNKNOWN.to_owned()])
        };
        for speaker in speakers {
            let v = by_speaker.entry(speaker).or_default();
            if let Some(last) = v.last_mut() {
                if start <= last.1 {
                    last.1 = last.1.max(end);
                    continue;
                }
            }
            v.push((start, end));
        }
    }
    let mut turns: Vec<_> = by_speaker
        .into_iter()
        .flat_map(|(speaker, v)| {
            v.into_iter().map(move |(start, end)| SpeakerTurn {
                speaker: speaker.clone(),
                start,
                end,
            })
        })
        .collect();
    turns.sort_by(|a, b| a.start.total_cmp(&b.start).then(a.speaker.cmp(&b.speaker)));
    primary.turns = turns;
    primary
        .known_speakers
        .as_mut()
        .unwrap()
        .method
        .push_str("+cross-window-consensus-offset-60s-v1");
    primary.validate(duration).map_err(|_| bad())?;
    Ok(primary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vara_core::KnownSpeakers;
    fn turn(s: &str, a: f64, b: f64) -> SpeakerTurn {
        SpeakerTurn {
            speaker: s.into(),
            start: a,
            end: b,
        }
    }
    fn doc(turns: Vec<SpeakerTurn>) -> Diarization {
        Diarization {
            model: "Who".into(),
            version: "test".into(),
            raw_turns: Some(turns.clone()),
            turns,
            known_speakers: Some(KnownSpeakers {
                count: 2,
                method: "test".into(),
                cosine_threshold: 0.65,
            }),
        }
    }
    #[test]
    fn permutation_overlap_disagreement_gaps_and_unknown_preserve_primary() {
        let a = doc(vec![
            turn("a", 0., 4.),
            turn("b", 4., 8.),
            turn("a", 8., 10.),
            turn("b", 8., 10.),
            turn("a", 10., 12.),
            turn(UNKNOWN, 12., 13.),
            turn("a", 13., 14.),
            turn("a", 14., 15.),
            turn("b", 14., 15.),
        ]);
        let b = doc(vec![
            turn("y", 60., 64.),
            turn("x", 64., 68.),
            turn("y", 68., 70.),
            turn("x", 68., 70.),
            turn("x", 70., 71.),
            turn(UNKNOWN, 71., 72.),
            turn("x", 72., 73.),
            turn("y", 74., 75.),
        ]);
        let raw = serde_json::to_value(&a.raw_turns).unwrap();
        let r = combine(a, b, 15., &Cancellation::default()).unwrap();
        assert_eq!(serde_json::to_value(&r.raw_turns).unwrap(), raw);
        assert!(
            r.turns
                .iter()
                .any(|t| t.speaker == "a" && t.start == 8. && t.end == 10.)
        );
        assert!(
            r.turns
                .iter()
                .any(|t| t.speaker == "b" && t.start == 4. && t.end == 10.)
        );
        assert!(
            r.turns
                .iter()
                .any(|t| t.speaker == UNKNOWN && t.start == 10. && t.end == 15.)
        );
    }
    #[test]
    fn shifted_ulp_does_not_create_uncertainty() {
        let a = doc(vec![turn("a", 0., 1.), turn("b", 1., 2.)]);
        let b = doc(vec![
            turn("x", 60., 61. + 1e-12),
            turn("y", 61. + 1e-12, 62.),
        ]);
        let r = combine(a, b, 2., &Cancellation::default()).unwrap();
        assert_eq!(r.turns.len(), 2);
        assert!(r.turns.iter().all(|t| t.speaker != UNKNOWN));
        assert!(r.turns.iter().any(|t| t.speaker == "a" && t.end == 1.));
    }
    #[test]
    fn ambiguous_or_nonpositive_mapping_is_rejected() {
        assert!(assignment(&[vec![1., 1.], vec![1., 1.]]).is_err());
        assert!(assignment(&[vec![1., 0.], vec![1., 0.]]).is_err());
        assert_eq!(
            assignment(&[vec![0., 2.], vec![3., 0.]]).unwrap(),
            vec![1, 0]
        );
    }
    #[test]
    fn cancellation_is_preserved() {
        let c = Cancellation::default();
        c.cancel();
        let a = doc(vec![turn("a", 0., 1.), turn("b", 1., 2.)]);
        let b = doc(vec![turn("x", 60., 61.), turn("y", 61., 62.)]);
        assert_eq!(combine(a, b, 2., &c).unwrap_err(), "Annulleret.");
    }
}
