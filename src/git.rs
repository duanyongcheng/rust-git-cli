use anyhow::{Context, Result};
use git2::{DiffOptions, Repository, StatusOptions};

pub struct GitRepo {
    repo: Repository,
}

impl GitRepo {
    pub fn open(path: &std::path::Path) -> Result<Self> {
        let repo = Repository::open(path).context("Failed to open repository")?;
        Ok(Self { repo })
    }

    pub fn get_status(&self) -> Result<GitStatus> {
        let mut status_opts = StatusOptions::new();
        status_opts.include_untracked(true);
        status_opts.include_ignored(false);

        let statuses = self
            .repo
            .statuses(Some(&mut status_opts))
            .context("Failed to get repository status")?;

        let mut modified_files = Vec::new();
        let mut new_files = Vec::new();
        let mut deleted_files = Vec::new();
        let mut renamed_files = Vec::new();

        for entry in statuses.iter() {
            let status = entry.status();
            let path = entry.path().unwrap_or("unknown").to_string();

            if status.is_wt_modified() || status.is_index_modified() {
                modified_files.push(path);
            } else if status.is_wt_new() || status.is_index_new() {
                new_files.push(path);
            } else if status.is_wt_deleted() || status.is_index_deleted() {
                deleted_files.push(path);
            } else if status.is_wt_renamed() || status.is_index_renamed() {
                renamed_files.push(path);
            }
        }

        Ok(GitStatus {
            is_clean: statuses.is_empty(),
            modified_files,
            new_files,
            deleted_files,
            renamed_files,
        })
    }

    pub fn get_branch_info(&self) -> Result<BranchInfo> {
        match self.repo.head() {
            Ok(head) => {
                if head.is_branch() {
                    let branch_name = head.shorthand().unwrap_or("unknown").to_string();

                    let tracking_info = if let Ok(upstream) =
                        self.repo.branch_upstream_name(head.name().unwrap())
                    {
                        let upstream_str = upstream.as_str().unwrap_or("unknown");
                        let (ahead, behind) = self.repo.graph_ahead_behind(
                            head.target().unwrap(),
                            self.repo.refname_to_id(upstream_str)?,
                        )?;
                        Some(TrackingInfo {
                            upstream: upstream_str.to_string(),
                            ahead,
                            behind,
                        })
                    } else {
                        None
                    };

                    Ok(BranchInfo {
                        name: Some(branch_name),
                        is_detached: false,
                        tracking_info,
                    })
                } else {
                    Ok(BranchInfo {
                        name: None,
                        is_detached: true,
                        tracking_info: None,
                    })
                }
            }
            Err(e) => {
                if e.code() == git2::ErrorCode::UnbornBranch {
                    Ok(BranchInfo {
                        name: Some("unborn".to_string()),
                        is_detached: false,
                        tracking_info: None,
                    })
                } else {
                    Err(e.into())
                }
            }
        }
    }

    pub fn get_diff(&self, staged: bool) -> Result<String> {
        let mut diff_opts = DiffOptions::new();
        diff_opts.include_untracked(true);

        let diff = if staged {
            // Get staged changes (index vs HEAD)
            match self.repo.head() {
                Ok(head) => {
                    let tree = head.peel_to_tree()?;
                    let mut index = self.repo.index()?;
                    // Force reload index from disk in case it was modified externally (e.g., git add)
                    index.read(true)?;
                    let index_tree = self.repo.find_tree(index.write_tree()?)?;
                    self.repo.diff_tree_to_tree(
                        Some(&tree),
                        Some(&index_tree),
                        Some(&mut diff_opts),
                    )?
                }
                Err(e) if e.code() == git2::ErrorCode::UnbornBranch => {
                    // No commits yet, compare index to empty tree
                    let mut index = self.repo.index()?;
                    // Force reload index from disk in case it was modified externally (e.g., git add)
                    index.read(true)?;
                    let index_tree = self.repo.find_tree(index.write_tree()?)?;
                    self.repo
                        .diff_tree_to_tree(None, Some(&index_tree), Some(&mut diff_opts))?
                }
                Err(e) => return Err(e.into()),
            }
        } else {
            // Get unstaged changes (working directory vs index)
            self.repo
                .diff_index_to_workdir(None, Some(&mut diff_opts))?
        };

        let mut diff_text = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            use git2::DiffLineType::*;
            let prefix = match line.origin_value() {
                Addition => "+",
                Deletion => "-",
                Context => " ",
                _ => "",
            };
            let content = std::str::from_utf8(line.content()).unwrap_or("");
            diff_text.push_str(&format!("{}{}", prefix, content));
            true
        })?;

        Ok(diff_text)
    }

    pub fn get_combined_diff(&self) -> Result<String> {
        let staged = self.get_diff(true)?;
        let unstaged = self.get_diff(false)?;

        let mut combined = String::new();

        if !staged.is_empty() {
            combined.push_str("=== STAGED CHANGES ===\n\n");
            combined.push_str(&staged);
        }

        if !unstaged.is_empty() {
            if !combined.is_empty() {
                combined.push_str("\n\n");
            }
            combined.push_str("=== UNSTAGED CHANGES ===\n\n");
            combined.push_str(&unstaged);
        }

        Ok(combined)
    }
}

pub struct GitStatus {
    pub is_clean: bool,
    pub modified_files: Vec<String>,
    pub new_files: Vec<String>,
    pub deleted_files: Vec<String>,
    pub renamed_files: Vec<String>,
}

impl GitStatus {
    pub fn total_changes(&self) -> usize {
        self.modified_files.len()
            + self.new_files.len()
            + self.deleted_files.len()
            + self.renamed_files.len()
    }
}

pub struct BranchInfo {
    pub name: Option<String>,
    pub is_detached: bool,
    pub tracking_info: Option<TrackingInfo>,
}

pub struct TrackingInfo {
    pub upstream: String,
    pub ahead: usize,
    pub behind: usize,
}

#[allow(dead_code)]
pub struct CommitInfo {
    pub id: String,
    pub short_id: String,
    pub author: String,
    pub email: String,
    pub time: chrono::DateTime<chrono::Local>,
    pub summary: String,
    pub message: String,
}

pub struct LogOptions {
    pub count: usize,
    pub grep: Option<String>,
    pub author: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
}

impl GitRepo {
    pub fn get_commits(&self, options: &LogOptions) -> Result<Vec<CommitInfo>> {
        use chrono::{Local, TimeZone};

        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let mut commits = Vec::new();

        for oid in revwalk {
            if commits.len() >= options.count {
                break;
            }

            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;

            let author = commit.author();
            let author_name = author.name().unwrap_or("unknown").to_string();
            let author_email = author.email().unwrap_or("").to_string();
            let summary = commit.summary().unwrap_or("").to_string();
            let message = commit.message().unwrap_or("").to_string();

            // Parse time
            let time_secs = commit.time().seconds();
            let time = Local
                .timestamp_opt(time_secs, 0)
                .single()
                .unwrap_or_else(Local::now);

            // Apply filters
            if let Some(ref grep_pattern) = options.grep {
                if !message
                    .to_lowercase()
                    .contains(&grep_pattern.to_lowercase())
                {
                    continue;
                }
            }

            if let Some(ref author_filter) = options.author {
                if !author_name
                    .to_lowercase()
                    .contains(&author_filter.to_lowercase())
                    && !author_email
                        .to_lowercase()
                        .contains(&author_filter.to_lowercase())
                {
                    continue;
                }
            }

            // Parse since/until dates
            if let Some(ref since_str) = options.since {
                if let Ok(since_date) = parse_date_string(since_str) {
                    if time < since_date {
                        continue;
                    }
                }
            }

            if let Some(ref until_str) = options.until {
                if let Ok(until_date) = parse_date_string(until_str) {
                    if time > until_date {
                        continue;
                    }
                }
            }

            commits.push(CommitInfo {
                id: oid.to_string(),
                short_id: oid.to_string()[..7].to_string(),
                author: author_name,
                email: author_email,
                time,
                summary,
                message,
            });
        }

        Ok(commits)
    }
}

fn parse_date_string(date_str: &str) -> Result<chrono::DateTime<chrono::Local>> {
    use chrono::{Local, NaiveDate};

    // Try parsing as YYYY-MM-DD
    if let Ok(naive_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        let datetime = naive_date.and_hms_opt(0, 0, 0).unwrap();
        return Ok(datetime.and_local_timezone(Local).unwrap());
    }

    // Try parsing relative dates like "1 week ago", "2 days ago"
    let parts: Vec<&str> = date_str.split_whitespace().collect();
    if parts.len() >= 2 {
        if let Ok(num) = parts[0].parse::<i64>() {
            let now = Local::now();
            let duration = match parts[1].trim_end_matches('s') {
                "day" => chrono::Duration::days(num),
                "week" => chrono::Duration::weeks(num),
                "month" => chrono::Duration::days(num * 30),
                "year" => chrono::Duration::days(num * 365),
                _ => return Err(anyhow::anyhow!("Unknown date format: {}", date_str)),
            };
            return Ok(now - duration);
        }
    }

    Err(anyhow::anyhow!("Could not parse date: {}", date_str))
}
