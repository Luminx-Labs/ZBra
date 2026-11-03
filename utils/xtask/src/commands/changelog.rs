use crate::context::Context;
use anyhow::{Context as _, Result};
use std::{collections::BTreeMap, fs};

const LOG_FORMAT: &str = "%H%x1f%h%x1f%aI%x1f%s%x1f%b%x1f%D%x1e";

#[derive(Clone, Debug, PartialEq, Eq)]
struct Commit {
    hash: String,
    short_hash: String,
    date: String,
    subject: String,
    body: String,
    tags: Vec<String>,
}

#[derive(Debug, Default)]
struct Section {
    title: String,
    date: Option<String>,
    commits: Vec<Commit>,
}

pub(super) fn run(context: &Context) -> Result<()> {
    let format_arg = format!("--format={LOG_FORMAT}");
    let output = context.output(
        "git",
        &["log", "--no-merges", "--decorate=short", &format_arg],
    )?;
    if !output.status.success() {
        anyhow::bail!("git log failed with status {}", output.status);
    }

    let log = String::from_utf8(output.stdout).context("git returned non-UTF-8 output")?;
    let commits = parse_commits(&log)?;
    let remote = context
        .output("git", &["config", "--get", "remote.origin.url"])
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|remote| normalize_remote(remote.trim()));
    let document = render(&commits, remote.as_deref());

    let path = context.path("CHANGELOG.md");
    fs::write(&path, document).with_context(|| format!("failed to write {}", path.display()))?;
    crate::success!("wrote {}", path.display());
    Ok(())
}

fn parse_commits(log: &str) -> Result<Vec<Commit>> {
    log.split('\u{1e}')
        .filter(|record| !record.trim().is_empty())
        .map(|record| {
            let fields: Vec<_> = record.trim().split('\u{1f}').collect();
            if fields.len() != 6 {
                anyhow::bail!("git log returned an unexpected record");
            }
            Ok(Commit {
                hash: fields[0].to_owned(),
                short_hash: fields[1].to_owned(),
                date: fields[2].chars().take(10).collect(),
                subject: fields[3].trim().to_owned(),
                body: fields[4].trim().to_owned(),
                tags: fields[5]
                    .split(',')
                    .filter_map(|decoration| decoration.trim().strip_prefix("tag: "))
                    .map(str::to_owned)
                    .collect(),
            })
        })
        .collect()
}

fn render(commits: &[Commit], remote: Option<&str>) -> String {
    let mut sections = Vec::new();
    let mut current = Section {
        title: "Unreleased".to_owned(),
        ..Section::default()
    };

    for commit in commits {
        if !commit.tags.is_empty() {
            if !current.commits.is_empty() {
                sections.push(current);
            }
            current = Section {
                title: commit.tags[0].clone(),
                date: Some(commit.date.clone()),
                ..Section::default()
            };
        }
        if !is_release_commit(commit) {
            current.commits.push(commit.clone());
        }
    }
    if !current.commits.is_empty() {
        sections.push(current);
    }

    let mut document = String::from("# Changelog\n\n");
    document.push_str("All notable changes to this project are documented here.\n\n");
    for section in sections {
        document.push_str("## ");
        document.push_str(&section.title);
        if let Some(date) = section.date {
            document.push_str(" - ");
            document.push_str(&date);
        }
        document.push_str("\n\n");
        render_categories(&mut document, &section.commits, remote);
    }
    if document == "# Changelog\n\nAll notable changes to this project are documented here.\n\n" {
        document.push_str("No changes have been recorded yet.\n");
    }
    document.truncate(document.trim_end().len());
    document.push('\n');
    document
}

fn render_categories(document: &mut String, commits: &[Commit], remote: Option<&str>) {
    let mut categories: BTreeMap<&str, Vec<&Commit>> = BTreeMap::new();
    for commit in commits {
        let category = category(commit);
        categories.entry(category).or_default().push(commit);
    }

    for category in ["Added", "Changed", "Fixed", "Documentation", "Other"] {
        let Some(commits) = categories.remove(category) else {
            continue;
        };
        document.push_str("### ");
        document.push_str(category);
        document.push_str("\n\n");
        for commit in commits {
            document.push_str("- ");
            if commit.subject.contains('!') || commit.body.contains("BREAKING CHANGE") {
                document.push_str("**Breaking:** ");
            }
            document.push_str(commit_description(&commit.subject));
            document.push_str(" ([");
            document.push_str(&commit.short_hash);
            document.push_str("](");
            if let Some(remote) = remote {
                document.push_str(remote);
                document.push_str("/commit/");
                document.push_str(&commit.hash);
            } else {
                document.push('#');
                document.push_str(&commit.short_hash);
            }
            document.push_str(")\n");
        }
        document.push('\n');
    }
}

fn category(commit: &Commit) -> &'static str {
    match commit.subject.split_once(':').map(|(prefix, _)| prefix) {
        Some(prefix) if prefix.starts_with("feat") => "Added",
        Some(prefix) if prefix.starts_with("fix") || prefix.starts_with("perf") => "Fixed",
        Some("docs") => "Documentation",
        Some("refactor" | "style" | "test" | "build" | "ci") => {
            "Changed"
        }
        _ => "Other",
    }
}

fn commit_description(subject: &str) -> &str {
    subject
        .split_once(':')
        .map_or(subject, |(_, description)| description.trim())
        .trim_start_matches('!')
        .trim()
}

fn is_release_commit(commit: &Commit) -> bool {
    commit.subject.starts_with("chore(release):")
}

fn normalize_remote(remote: &str) -> Option<String> {
    let remote = remote.strip_suffix(".git").unwrap_or(remote);
    let normalized = if let Some(remote) = remote.strip_prefix("git@") {
        format!("https://{}", remote.replacen(':', "/", 1))
    } else if let Some(remote) = remote.strip_prefix("ssh://git@") {
        format!("https://{remote}")
    } else {
        remote.to_owned()
    };
    (!normalized.is_empty()).then_some(normalized.trim_end_matches('/').to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_conventional_commit_and_tag() {
        let commits = parse_commits(
            "abc\u{1f}abc\u{1f}2026-01-02T00:00:00+00:00\u{1f}feat(chess): add moves\u{1f}\u{1f}tag: v1.0.0, HEAD -> main\u{1e}",
        )
        .unwrap();

        assert_eq!(commits[0].tags, ["v1.0.0"]);
        assert_eq!(category(&commits[0]), "Added");
        assert_eq!(commit_description(&commits[0].subject), "add moves");
    }

    #[test]
    fn renders_release_sections_and_skips_release_commit() {
        let commits = vec![
            Commit {
                hash: "a".into(),
                short_hash: "a".into(),
                date: "2026-01-02".into(),
                subject: "feat: new thing".into(),
                body: String::new(),
                tags: vec![],
            },
            Commit {
                hash: "b".into(),
                short_hash: "b".into(),
                date: "2026-01-01".into(),
                subject: "chore(release): v1.0.0".into(),
                body: String::new(),
                tags: vec!["v1.0.0".into()],
            },
            Commit {
                hash: "c".into(),
                short_hash: "c".into(),
                date: "2025-12-31".into(),
                subject: "fix: old bug".into(),
                body: String::new(),
                tags: vec![],
            },
        ];
        let document = render(&commits, Some("https://example.com/zbra"));

        assert!(document.contains("## Unreleased"));
        assert!(document.contains("## v1.0.0 - 2026-01-01"));
        assert!(!document.contains("add moves"));
        assert!(!document.contains("chore(release)"));
        assert!(document.contains("https://example.com/zbra/commit/a"));
    }

    #[test]
    fn normalizes_common_git_remote_formats() {
        assert_eq!(
            normalize_remote("git@github.com:owner/repo.git"),
            Some("https://github.com/owner/repo".into())
        );
        assert_eq!(
            normalize_remote("https://github.com/owner/repo.git"),
            Some("https://github.com/owner/repo".into())
        );
    }
}
