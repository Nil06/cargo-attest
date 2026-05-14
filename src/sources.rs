//! External source adapters: GitHub releases, crates.io, sigstore.

pub mod github {
    use std::time::Duration;

    use anyhow::{anyhow, Context, Result};
    use serde::Deserialize;

    const USER_AGENT: &str = concat!("cargo-attest/", env!("CARGO_PKG_VERSION"));
    const API_BASE: &str = "https://api.github.com";

    #[derive(Debug, Clone)]
    pub struct ReleaseMeta {
        pub tag: String,
        /// Commit SHA the tag points at (resolved separately from the release object).
        pub commit_sha: Option<String>,
        /// Free-text body of the release — often where maintainers paste checksums.
        pub body: String,
        pub assets: Vec<Asset>,
    }

    #[derive(Debug, Clone)]
    pub struct Asset {
        pub name: String,
        pub download_url: String,
        pub size: u64,
    }

    #[derive(Deserialize)]
    struct ReleaseJson {
        tag_name: String,
        #[serde(default)]
        body: Option<String>,
        assets: Vec<AssetJson>,
    }

    #[derive(Deserialize)]
    struct AssetJson {
        name: String,
        browser_download_url: String,
        size: u64,
    }

    #[derive(Deserialize)]
    struct TagRefJson {
        object: TagRefObject,
    }

    #[derive(Deserialize)]
    struct TagRefObject {
        sha: String,
        #[serde(rename = "type")]
        ref_type: String,
    }

    #[derive(Deserialize)]
    struct AnnotatedTagJson {
        object: TagRefObject,
    }

    fn client() -> Result<reqwest::blocking::Client> {
        let mut builder = reqwest::blocking::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30));
        if let Ok(token) = std::env::var("GH_TOKEN").or_else(|_| std::env::var("GITHUB_TOKEN")) {
            let mut headers = reqwest::header::HeaderMap::new();
            let val = reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))
                .context("invalid GH_TOKEN header value")?;
            headers.insert(reqwest::header::AUTHORIZATION, val);
            builder = builder.default_headers(headers);
        }
        builder.build().context("building HTTP client")
    }

    /// Fetch a release by tag and resolve the underlying commit SHA.
    pub fn fetch_release(repo: &str, tag: &str) -> Result<ReleaseMeta> {
        if !repo.contains('/') {
            return Err(anyhow!("expected owner/repo, got {repo:?}"));
        }
        let c = client()?;
        let url = format!("{API_BASE}/repos/{repo}/releases/tags/{tag}");
        let resp = c.get(&url).send().with_context(|| format!("GET {url}"))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(anyhow!("GitHub release lookup failed: {status}"));
        }
        let rel: ReleaseJson = resp.json().context("parsing release JSON")?;

        let commit_sha = resolve_tag_commit(&c, repo, tag).ok();

        Ok(ReleaseMeta {
            tag: rel.tag_name,
            commit_sha,
            body: rel.body.unwrap_or_default(),
            assets: rel
                .assets
                .into_iter()
                .map(|a| Asset {
                    name: a.name,
                    download_url: a.browser_download_url,
                    size: a.size,
                })
                .collect(),
        })
    }

    /// Resolve a tag ref to a commit SHA, dereferencing annotated tags once.
    fn resolve_tag_commit(c: &reqwest::blocking::Client, repo: &str, tag: &str) -> Result<String> {
        let url = format!("{API_BASE}/repos/{repo}/git/refs/tags/{tag}");
        let r: TagRefJson = c.get(&url).send()?.error_for_status()?.json()?;
        if r.object.ref_type == "commit" {
            return Ok(r.object.sha);
        }
        // Annotated tag: dereference once.
        let url = format!("{API_BASE}/repos/{repo}/git/tags/{}", r.object.sha);
        let t: AnnotatedTagJson = c.get(&url).send()?.error_for_status()?.json()?;
        Ok(t.object.sha)
    }

    /// Download an asset to a destination path. Streams to disk.
    pub fn download_asset(url: &str, dest: &std::path::Path) -> Result<u64> {
        let c = client()?;
        let mut resp = c.get(url).send()?.error_for_status()?;
        let mut file =
            std::fs::File::create(dest).with_context(|| format!("creating {}", dest.display()))?;
        let n = std::io::copy(&mut resp, &mut file).context("streaming asset to disk")?;
        Ok(n)
    }

    /// Download a small text asset such as `<artifact>.sha256`.
    pub fn download_asset_text(url: &str) -> Result<String> {
        let c = client()?;
        c.get(url)
            .send()?
            .error_for_status()?
            .text()
            .context("reading text asset")
    }
}

pub mod crates_io {
    use anyhow::{bail, Result};

    /// Stub: returns Err until v0.4.
    pub fn fetch_crate_meta(_name: &str, _version: &str) -> Result<CrateMeta> {
        bail!("crates_io::fetch_crate_meta not implemented yet (planned v0.4)")
    }

    #[derive(Debug)]
    pub struct CrateMeta {
        pub name: String,
        pub version: String,
        pub repository: Option<String>,
        pub checksum: String,
    }
}

#[allow(dead_code)]
pub mod sigstore {
    use anyhow::{bail, Result};

    /// Stub: returns Err until v0.5.
    pub fn verify_cosign_signature(_artifact_sha256: &str, _signature_url: &str) -> Result<bool> {
        bail!("sigstore::verify_cosign_signature not implemented yet (planned v0.5)")
    }
}
