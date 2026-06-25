//! Workspace filesystem sandboxing.
//!
//! Confines agent file operations to their workspace directory.
//! Prevents path traversal, symlink escapes, and access outside the sandbox.

use std::path::{Path, PathBuf};

/// Resolve a user-supplied path within a workspace sandbox.
///
/// - Rejects `..` components outright.
/// - Relative paths are joined with `workspace_root`.
/// - Absolute paths are checked against the workspace root after canonicalization.
/// - For new files: canonicalizes the parent directory and appends the filename.
/// - The final canonical path must start with the canonical workspace root.
pub fn resolve_sandbox_path(user_path: &str, workspace_root: &Path) -> Result<PathBuf, String> {
    resolve_sandbox_path_multi(user_path, workspace_root, &[])
}

/// Resolve a user-supplied path against the workspace sandbox **plus** a set of
/// additional allowed roots.
///
/// Relative paths are always resolved under `workspace_root` (the agent's
/// working area). Absolute paths are accepted if they canonicalize inside
/// `workspace_root` **or** any of `extra_roots`. This grants explicitly-declared
/// least-privilege access to directories mounted outside the workspace — e.g. a
/// coordinator agent's Logseq graph PVC at `/data/graphs/<agent>` — without
/// opening up the whole filesystem. `extra_roots` come from the agent manifest's
/// `[capabilities].file_roots`; an empty slice is identical to the plain
/// workspace sandbox.
pub fn resolve_sandbox_path_multi(
    user_path: &str,
    workspace_root: &Path,
    extra_roots: &[PathBuf],
) -> Result<PathBuf, String> {
    let path = Path::new(user_path);

    // Reject any `..` components
    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err("Path traversal denied: '..' components are forbidden".to_string());
        }
    }

    // Build the candidate path. Relative paths resolve under the workspace;
    // absolute paths are validated against every allowed root below.
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    };

    // Canonicalize the candidate (or its parent for new files)
    let canon_candidate = if candidate.exists() {
        candidate
            .canonicalize()
            .map_err(|e| format!("Failed to resolve path: {e}"))?
    } else {
        // For new files: canonicalize the parent and append the filename
        let parent = candidate
            .parent()
            .ok_or_else(|| "Invalid path: no parent directory".to_string())?;
        let filename = candidate
            .file_name()
            .ok_or_else(|| "Invalid path: no filename".to_string())?;
        let canon_parent = parent
            .canonicalize()
            .map_err(|e| format!("Failed to resolve parent directory: {e}"))?;
        canon_parent.join(filename)
    };

    // The candidate is allowed if it lives inside the workspace OR any of the
    // explicitly-granted extra roots. Roots that fail to canonicalize (e.g. an
    // unmounted path) are skipped rather than treated as an error.
    let allowed = std::iter::once(workspace_root)
        .chain(extra_roots.iter().map(|p| p.as_path()))
        .filter_map(|root| root.canonicalize().ok())
        .any(|canon_root| canon_candidate.starts_with(&canon_root));

    if !allowed {
        return Err(format!(
            "Access denied: path '{}' resolves outside the agent workspace and \
             any granted file_roots. Declare the directory under \
             [capabilities].file_roots in the agent manifest, or use the \
             mcp_filesystem_* tools if an MCP filesystem server is configured.",
            user_path
        ));
    }

    Ok(canon_candidate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_relative_path_inside_workspace() {
        let dir = TempDir::new().unwrap();
        let data_dir = dir.path().join("data");
        std::fs::create_dir_all(&data_dir).unwrap();
        std::fs::write(data_dir.join("test.txt"), "hello").unwrap();

        let result = resolve_sandbox_path("data/test.txt", dir.path());
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert!(resolved.starts_with(dir.path().canonicalize().unwrap()));
    }

    #[test]
    fn test_absolute_path_inside_workspace() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("file.txt"), "ok").unwrap();
        let abs_path = dir.path().join("file.txt");

        let result = resolve_sandbox_path(abs_path.to_str().unwrap(), dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_absolute_path_outside_workspace_blocked() {
        let dir = TempDir::new().unwrap();
        let outside = std::env::temp_dir().join("outside_test.txt");
        std::fs::write(&outside, "nope").unwrap();

        let result = resolve_sandbox_path(outside.to_str().unwrap(), dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Access denied"));

        let _ = std::fs::remove_file(&outside);
    }

    #[test]
    fn test_dotdot_component_blocked() {
        let dir = TempDir::new().unwrap();
        let result = resolve_sandbox_path("../../../etc/passwd", dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Path traversal denied"));
    }

    #[test]
    fn test_nonexistent_file_with_valid_parent() {
        let dir = TempDir::new().unwrap();
        let data_dir = dir.path().join("data");
        std::fs::create_dir_all(&data_dir).unwrap();

        let result = resolve_sandbox_path("data/new_file.txt", dir.path());
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert!(resolved.starts_with(dir.path().canonicalize().unwrap()));
        assert!(resolved.ends_with("new_file.txt"));
    }

    #[test]
    fn test_absolute_path_in_extra_root_allowed() {
        let workspace = TempDir::new().unwrap();
        let graph = TempDir::new().unwrap();
        let pages = graph.path().join("pages/world");
        std::fs::create_dir_all(&pages).unwrap();
        let target = pages.join("dashboard.md");

        // New file under a granted extra root: allowed.
        let result = resolve_sandbox_path_multi(
            target.to_str().unwrap(),
            workspace.path(),
            &[graph.path().to_path_buf()],
        );
        assert!(result.is_ok(), "{:?}", result);
        assert!(result
            .unwrap()
            .starts_with(graph.path().canonicalize().unwrap()));
    }

    #[test]
    fn test_absolute_path_outside_extra_roots_blocked() {
        let workspace = TempDir::new().unwrap();
        let graph = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        std::fs::write(outside.path().join("secret.txt"), "nope").unwrap();

        let result = resolve_sandbox_path_multi(
            outside.path().join("secret.txt").to_str().unwrap(),
            workspace.path(),
            &[graph.path().to_path_buf()],
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Access denied"));
    }

    #[test]
    fn test_relative_path_still_workspace_bound_with_extra_roots() {
        let workspace = TempDir::new().unwrap();
        let graph = TempDir::new().unwrap();
        std::fs::create_dir_all(workspace.path().join("notes")).unwrap();
        // A relative path resolves under the workspace, never the extra root.
        let result = resolve_sandbox_path_multi(
            "notes/today.md",
            workspace.path(),
            &[graph.path().to_path_buf()],
        );
        assert!(result.is_ok(), "{:?}", result);
        assert!(result
            .unwrap()
            .starts_with(workspace.path().canonicalize().unwrap()));
    }

    #[cfg(unix)]
    #[test]
    fn test_symlink_escape_blocked() {
        let dir = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        std::fs::write(outside.path().join("secret.txt"), "secret").unwrap();

        // Create a symlink inside the workspace pointing outside
        let link_path = dir.path().join("escape");
        std::os::unix::fs::symlink(outside.path(), &link_path).unwrap();

        let result = resolve_sandbox_path("escape/secret.txt", dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Access denied"));
    }
}
