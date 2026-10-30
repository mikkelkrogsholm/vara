//! Known-count grouping over SDK lane embeddings. Embeddings never leave the worker.
use crate::Result;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use vara_core::{Diarization, KnownSpeakers, SpeakerTurn};

#[derive(Deserialize)]
struct Payload {
    cosine_threshold: f64,
    piece_seconds: f64,
    lanes: Vec<Lane>,
}
#[derive(Deserialize)]
struct Lane {
    piece: u32,
    final_speaker: u32,
    embedding: Option<Vec<f64>>,
    turns: Vec<Interval>,
}
#[derive(Deserialize)]
struct Interval {
    start: f64,
    end: f64,
}
fn bad() -> String {
    "Who kunne ikke afklare det valgte antal talere. Tidligere resultat er bevaret.".into()
}
fn unit(v: &[f64]) -> Result<Vec<f64>> {
    if v.is_empty() || v.len() > 2048 || v.iter().any(|x| !x.is_finite()) {
        return Err(bad());
    }
    let norm = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    if !norm.is_finite() || norm < 1e-8 {
        return Err(bad());
    }
    Ok(v.iter().map(|x| x / norm).collect())
}
fn union(mut v: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    v.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    let mut out: Vec<(f64, f64)> = Vec::new();
    for (a, b) in v {
        if let Some(last) = out.last_mut() {
            if a <= last.1 + 1e-6 {
                last.1 = last.1.max(b);
                continue;
            }
        }
        out.push((a, b));
    }
    out
}
fn equal_union(a: Vec<(f64, f64)>, b: Vec<(f64, f64)>) -> bool {
    let a = union(a);
    let b = union(b);
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(x, y)| (x.0 - y.0).abs() < 1e-5 && (x.1 - y.1).abs() < 1e-5)
}
pub(super) fn group(
    mut raw: Diarization,
    bytes: &[u8],
    count: u32,
    duration: f64,
) -> Result<Diarization> {
    if !(1..=16).contains(&count) || bytes.len() > 32_000_000 {
        return Err(bad());
    }
    let payload: Payload = serde_json::from_slice(bytes).map_err(|_| bad())?;
    if payload.lanes.is_empty()
        || payload.lanes.len() > 10_000
        || !payload.piece_seconds.is_finite()
        || !(1.0..=600.0).contains(&payload.piece_seconds)
        || !payload.cosine_threshold.is_finite()
        || !(0.0..=1.0).contains(&payload.cosine_threshold)
    {
        return Err(bad());
    }
    let mut groups: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    let mut normalized = Vec::with_capacity(payload.lanes.len());
    let mut dimension = None;
    let mut total_turns = 0usize;
    for (i, lane) in payload.lanes.iter().enumerate() {
        if lane.turns.is_empty() {
            return Err(bad());
        }
        total_turns = total_turns.checked_add(lane.turns.len()).ok_or_else(bad)?;
        if total_turns > 1_000_000
            || lane.turns.iter().any(|t| {
                !t.start.is_finite()
                    || !t.end.is_finite()
                    || t.start < 0.
                    || t.end <= t.start
                    || t.end > duration + 0.1
                    || t.start < lane.piece as f64 * payload.piece_seconds - 0.1
                    || t.end > (lane.piece as f64 + 1.) * payload.piece_seconds + 0.1
            })
        {
            return Err(bad());
        }
        let emb = lane.embedding.as_deref().map(unit).transpose()?;
        if let Some(v) = &emb {
            if dimension.is_some_and(|n| n != v.len()) {
                return Err(bad());
            }
            dimension = Some(v.len());
        }
        normalized.push(emb);
        groups.entry(lane.final_speaker).or_default().push(i);
    }
    // Validate lane times against the exact SDK output, per SDK label, before grouping.
    let mut raw_groups: BTreeMap<u32, Vec<(f64, f64)>> = BTreeMap::new();
    for t in &raw.turns {
        let id = t
            .speaker
            .strip_prefix("who-")
            .and_then(|s| s.parse().ok())
            .ok_or_else(bad)?;
        raw_groups.entry(id).or_default().push((t.start, t.end));
    }
    if groups.len() != raw_groups.len() {
        return Err(bad());
    }
    for (id, indices) in &groups {
        let intervals = indices
            .iter()
            .flat_map(|i| payload.lanes[*i].turns.iter().map(|t| (t.start, t.end)))
            .collect();
        if !equal_union(intervals, raw_groups.remove(id).ok_or_else(bad)?) {
            return Err(bad());
        }
    }
    let mut ranked: Vec<_> = groups
        .iter()
        .filter_map(|(&id, indices)| {
            let voiced: Vec<_> = indices
                .iter()
                .copied()
                .filter(|i| normalized[*i].is_some())
                .collect();
            let pieces: BTreeSet<_> = voiced.iter().map(|i| payload.lanes[*i].piece).collect();
            if pieces.len() < 2 {
                return None;
            }
            let times: Vec<_> = indices
                .iter()
                .flat_map(|i| payload.lanes[*i].turns.iter().map(|t| (t.start, t.end)))
                .collect();
            let active = union(times.clone()).iter().map(|(a, b)| b - a).sum::<f64>();
            let first = times.iter().map(|(a, _)| *a).fold(f64::INFINITY, f64::min);
            Some((id, pieces.len(), active, first, voiced))
        })
        .collect();
    ranked.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then(b.2.total_cmp(&a.2))
            .then(a.3.total_cmp(&b.3))
    });
    if ranked.len() < count as usize {
        return Err(bad());
    }
    ranked.truncate(count as usize);
    let dim = dimension.ok_or_else(bad)?;
    // Keep sums by piece so an anchor cannot validate itself with its own voice.
    let mut references = Vec::new();
    for (id, _, _, _, voiced) in ranked {
        let mut total = vec![0.; dim];
        let mut pieces: BTreeMap<u32, Vec<f64>> = BTreeMap::new();
        for i in voiced {
            let piece = pieces
                .entry(payload.lanes[i].piece)
                .or_insert_with(|| vec![0.; dim]);
            for ((a, p), b) in total
                .iter_mut()
                .zip(piece)
                .zip(normalized[i].as_ref().unwrap())
            {
                *a += b;
                *p += b;
            }
        }
        let prototype = unit(&total)?;
        references.push((id, prototype, total, pieces));
    }
    let mut order: Vec<_> = (0..payload.lanes.len()).collect();
    order.sort_by_key(|&i| payload.lanes[i].piece);
    let mut current_piece = None;
    let mut heldout = Vec::new();
    let mut turns = Vec::with_capacity(total_turns);
    for i in order {
        let lane = &payload.lanes[i];
        if current_piece != Some(lane.piece) {
            heldout.clear();
            for (_, _, total, pieces) in &references {
                let remaining: Vec<_> = match pieces.get(&lane.piece) {
                    Some(part) => total.iter().zip(part).map(|(a, b)| a - b).collect(),
                    None => total.clone(),
                };
                heldout.push(unit(&remaining).ok());
            }
            current_piece = Some(lane.piece);
        }
        let assigned = if let Some(emb) = &normalized[i] {
            if references
                .iter()
                .any(|(id, _, _, _)| *id == lane.final_speaker)
            {
                let matches: Vec<_> = references
                    .iter()
                    .zip(&heldout)
                    .filter_map(|((id, _, _, _), prototype)| {
                        prototype
                            .as_ref()
                            .filter(|p| {
                                emb.iter().zip(*p).map(|(a, b)| a * b).sum::<f64>()
                                    >= payload.cosine_threshold
                            })
                            .map(|_| *id)
                    })
                    .collect();
                if matches.len() == 1 && matches[0] == lane.final_speaker {
                    Some(lane.final_speaker)
                } else {
                    None
                }
            } else {
                let matches: Vec<_> = references
                    .iter()
                    .filter(|(_, prototype, _, _)| {
                        emb.iter().zip(prototype).map(|(a, b)| a * b).sum::<f64>()
                            >= payload.cosine_threshold
                    })
                    .collect();
                if matches.len() == 1 {
                    Some(matches[0].0)
                } else {
                    None
                }
            }
        } else {
            None
        };
        let speaker = assigned
            .map(|id| format!("who-{id}"))
            .unwrap_or_else(|| "who-uncertain".into());
        for t in &lane.turns {
            turns.push(SpeakerTurn {
                speaker: speaker.clone(),
                start: t.start,
                end: t.end,
            });
        }
    }
    // Coalesce same-speaker activity, avoiding double-counted coverage after mapping.
    let mut by_speaker: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    for t in turns {
        by_speaker
            .entry(t.speaker)
            .or_default()
            .push((t.start, t.end));
    }
    let mut turns: Vec<SpeakerTurn> = by_speaker
        .into_iter()
        .flat_map(|(speaker, intervals)| {
            union(intervals)
                .into_iter()
                .map(move |(start, end)| SpeakerTurn {
                    speaker: speaker.clone(),
                    start,
                    end,
                })
        })
        .collect();
    turns.sort_by(|a, b| a.start.total_cmp(&b.start).then(a.speaker.cmp(&b.speaker)));
    raw.raw_turns = Some(std::mem::replace(&mut raw.turns, turns));
    raw.known_speakers = Some(KnownSpeakers {
        count,
        method: "recurring-sdk-lanes-heldout-unique-cosine-v2".into(),
        cosine_threshold: payload.cosine_threshold,
    });
    raw.validate(duration).map_err(|_| bad())?;
    Ok(raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    fn fixture() -> (Diarization, Value) {
        let rows = vec![
            (8, 0, 0., 2., Some(vec![1., 0.])),
            (8, 1, 10., 12., Some(vec![1., 0.])),
            (3, 0, 2., 4., Some(vec![0., 1.])),
            (3, 1, 12., 14., Some(vec![0., 1.])),
            (11, 0, 4., 5., Some(vec![1., 0.])),
            (12, 0, 5., 6., Some(vec![1., 1.])),
            (13, 0, 6., 7., Some(vec![-1., 0.])),
            (14, 0, 7., 8., None),
            (15, 0, 4.5, 5.5, Some(vec![1., 0.])),
        ];
        let turns = rows
            .iter()
            .map(|(id, _, start, end, _)| SpeakerTurn {
                speaker: format!("who-{id}"),
                start: *start,
                end: *end,
            })
            .collect();
        let lanes:Vec<_>=rows.iter().map(|(id,piece,start,end,embedding)|json!({"piece":piece,"final_speaker":id,"embedding":embedding,"turns":[{"start":start,"end":end}]})).collect();
        (
            Diarization {
                model: "Who".into(),
                version: "test".into(),
                turns,
                raw_turns: None,
                known_speakers: None,
            },
            json!({"cosine_threshold":0.65,"piece_seconds":10.,"lanes":lanes}),
        )
    }
    #[test]
    fn unique_matches_and_uncertainty_preserve_all_intervals() {
        let (raw, payload) = fixture();
        let out = group(raw.clone(), payload.to_string().as_bytes(), 2, 20.).unwrap();
        assert_eq!(out.raw_turns.as_ref().unwrap().len(), raw.turns.len());
        assert!(
            out.turns
                .iter()
                .any(|t| t.speaker == "who-8" && t.start == 4. && t.end == 5.5)
        );
        assert!(
            out.turns
                .iter()
                .any(|t| t.speaker == "who-uncertain" && t.start == 5. && t.end == 8.)
        );
        assert!(
            out.turns
                .iter()
                .any(|t| t.speaker == "who-3" && t.start == 2.)
        );
        assert!(equal_union(
            raw.turns.iter().map(|t| (t.start, t.end)).collect(),
            out.turns.iter().map(|t| (t.start, t.end)).collect()
        ));
    }
    #[test]
    fn reference_selection_ignores_order_and_label_values_and_never_merges() {
        let (raw, mut payload) = fixture();
        let original = group(raw.clone(), payload.to_string().as_bytes(), 2, 20.).unwrap();
        payload["lanes"].as_array_mut().unwrap().reverse();
        let reversed = group(raw.clone(), payload.to_string().as_bytes(), 2, 20.).unwrap();
        assert_eq!(
            serde_json::to_value(original.turns).unwrap(),
            serde_json::to_value(reversed.turns).unwrap()
        );
        let mut relabelled = raw.clone();
        for turn in &mut relabelled.turns {
            turn.speaker = match turn.speaker.as_str() {
                "who-8" => "who-3".into(),
                "who-3" => "who-8".into(),
                _ => turn.speaker.clone(),
            };
        }
        let mut relabelled_payload = payload.clone();
        for lane in relabelled_payload["lanes"].as_array_mut().unwrap() {
            let id = lane["final_speaker"].as_u64().unwrap();
            if id == 8 {
                lane["final_speaker"] = json!(3);
            } else if id == 3 {
                lane["final_speaker"] = json!(8);
            }
        }
        let swapped = group(
            relabelled,
            relabelled_payload.to_string().as_bytes(),
            2,
            20.,
        )
        .unwrap();
        assert!(
            swapped
                .turns
                .iter()
                .any(|t| t.speaker == "who-3" && t.start == 4. && t.end == 5.5)
        );
        assert!(
            swapped
                .turns
                .iter()
                .any(|t| t.speaker == "who-8" && t.start == 2.)
        );
        assert!(group(raw.clone(), payload.to_string().as_bytes(), 3, 20.).is_err());
        payload["lanes"][0]["piece"] = json!(900);
        assert!(group(raw, payload.to_string().as_bytes(), 2, 20.).is_err());
    }
    #[test]
    fn wrong_reference_lane_is_uncertain_not_relabelled() {
        let (mut raw, mut payload) = fixture();
        raw.turns.push(SpeakerTurn {
            speaker: "who-8".into(),
            start: 20.,
            end: 21.,
        });
        payload["lanes"].as_array_mut().unwrap().push(json!({"piece":2,"final_speaker":8,"embedding":[1.,0.],"turns":[{"start":20.,"end":21.}]}));
        raw.turns.push(SpeakerTurn {
            speaker: "who-8".into(),
            start: 30.,
            end: 31.,
        });
        payload["lanes"].as_array_mut().unwrap().push(json!({"piece":3,"final_speaker":8,"embedding":[0.,1.],"turns":[{"start":30.,"end":31.}]}));
        let out = group(raw, payload.to_string().as_bytes(), 2, 40.).unwrap();
        assert!(
            out.turns
                .iter()
                .any(|t| t.speaker == "who-uncertain" && t.start == 30. && t.end == 31.)
        );
        assert!(
            !out.turns
                .iter()
                .any(|t| t.speaker == "who-3" && t.start == 30.)
        );
    }
    #[test]
    fn invalid_payload_and_count_rejected() {
        let (raw, mut payload) = fixture();
        assert!(group(raw.clone(), payload.to_string().as_bytes(), 0, 20.).is_err());
        payload["lanes"][0]["embedding"] = json!([0., 0.]);
        assert!(group(raw, payload.to_string().as_bytes(), 2, 20.).is_err());
    }
}
