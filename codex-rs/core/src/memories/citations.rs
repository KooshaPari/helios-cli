use codex_protocol::ThreadId;
<<<<<<< HEAD

pub fn get_thread_id_from_citations(citations: Vec<String>) -> Vec<ThreadId> {
    let mut result = Vec::new();
    for citation in citations {
        let mut ids_block = None;
        for (open, close) in [
            ("<thread_ids>", "</thread_ids>"),
            ("<rollout_ids>", "</rollout_ids>"),
        ] {
            if let Some((_, rest)) = citation.split_once(open)
                && let Some((ids, _)) = rest.split_once(close)
            {
                ids_block = Some(ids);
                break;
            }
        }

        if let Some(ids_block) = ids_block {
=======
use codex_protocol::memory_citation::MemoryCitation;
use codex_protocol::memory_citation::MemoryCitationEntry;
use std::collections::HashSet;

pub fn parse_memory_citation(citations: Vec<String>) -> Option<MemoryCitation> {
    let mut entries = Vec::new();
    let mut rollout_ids = Vec::new();
    let mut seen_rollout_ids = HashSet::new();

    for citation in citations {
        if let Some(entries_block) =
            extract_block(&citation, "<citation_entries>", "</citation_entries>")
        {
            entries.extend(
                entries_block
                    .lines()
                    .filter_map(parse_memory_citation_entry),
            );
        }

        if let Some(ids_block) = extract_ids_block(&citation) {
>>>>>>> upstream_main
            for id in ids_block
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
            {
<<<<<<< HEAD
                if let Ok(thread_id) = ThreadId::try_from(id) {
                    result.push(thread_id);
=======
                if seen_rollout_ids.insert(id.to_string()) {
                    rollout_ids.push(id.to_string());
>>>>>>> upstream_main
                }
            }
        }
    }
<<<<<<< HEAD
    result
}

#[cfg(test)]
mod tests {
    use super::get_thread_id_from_citations;
    use codex_protocol::ThreadId;
    use pretty_assertions::assert_eq;

    #[test]
    fn get_thread_id_from_citations_extracts_thread_ids() {
        let first = ThreadId::new();
        let second = ThreadId::new();

        let citations = vec![format!(
            "<memory_citation>\n<citation_entries>\nMEMORY.md:1-2|note=[x]\n</citation_entries>\n<thread_ids>\n{first}\nnot-a-uuid\n{second}\n</thread_ids>\n</memory_citation>"
        )];

        assert_eq!(get_thread_id_from_citations(citations), vec![first, second]);
    }

    #[test]
    fn get_thread_id_from_citations_supports_legacy_rollout_ids() {
        let thread_id = ThreadId::new();

        let citations = vec![format!(
            "<memory_citation>\n<rollout_ids>\n{thread_id}\n</rollout_ids>\n</memory_citation>"
        )];

        assert_eq!(get_thread_id_from_citations(citations), vec![thread_id]);
    }
}
=======

    if entries.is_empty() && rollout_ids.is_empty() {
        None
    } else {
        Some(MemoryCitation {
            entries,
            rollout_ids,
        })
    }
}

pub fn get_thread_id_from_citations(citations: Vec<String>) -> Vec<ThreadId> {
    let mut result = Vec::new();
    if let Some(memory_citation) = parse_memory_citation(citations) {
        for rollout_id in memory_citation.rollout_ids {
            if let Ok(thread_id) = ThreadId::try_from(rollout_id.as_str()) {
                result.push(thread_id);
            }
        }
    }
    result
}

fn parse_memory_citation_entry(line: &str) -> Option<MemoryCitationEntry> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    let (location, note) = line.rsplit_once("|note=[")?;
    let note = note.strip_suffix(']')?.trim().to_string();
    let (path, line_range) = location.rsplit_once(':')?;
    let (line_start, line_end) = line_range.split_once('-')?;

    Some(MemoryCitationEntry {
        path: path.trim().to_string(),
        line_start: line_start.trim().parse().ok()?,
        line_end: line_end.trim().parse().ok()?,
        note,
    })
}

fn extract_block<'a>(text: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let (_, rest) = text.split_once(open)?;
    let (body, _) = rest.split_once(close)?;
    Some(body)
}

fn extract_ids_block(text: &str) -> Option<&str> {
    extract_block(text, "<rollout_ids>", "</rollout_ids>")
        .or_else(|| extract_block(text, "<thread_ids>", "</thread_ids>"))
}

#[cfg(test)]
#[path = "citations_tests.rs"]
mod tests;
>>>>>>> upstream_main
