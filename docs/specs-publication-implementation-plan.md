# Publish the CityJSON Arrow and Parquet Specifications

Implementation plan retained for project history. The publishing target is
<https://specs.citymodel.3dgi.nl/> using GitHub Pages. The site combines a small
landing portal with the existing `cityjson-arrow` and `cityjson-parquet`
ProperDocs sites, applies CC BY 4.0 to specification prose and diagrams, credits
Balázs Dukai as author and licensor with 3DGI as affiliation, and keeps source
code under MIT OR Apache-2.0.

The Pages workflow builds all three sections with strict documentation checks,
assembles one artifact, validates canonical URLs, attribution, and internal
links, and deploys pushes to `main`. Initial publication is the experimental
`cityjson-arrow.package.v3alpha3` specification. Historical snapshots are
identified by specification tags; multi-version hosting is deferred until a
second specification version exists.

## Implementation

- Build the portal at `/`, Arrow documentation at `/arrow/`, and Parquet
  documentation at `/parquet/` into one GitHub Pages artifact.
- Publish only successful pushes to `main`; pull requests build and validate the
  same artifact without deploying it.
- Require all normative pages to display experimental status, format version,
  author/editor, 3DGI affiliation, CC BY 4.0, and source-repository metadata.
- Keep Balázs Dukai as copyright holder and licensor. Use 3DGI only as
  professional affiliation and project identity.
- Keep generated site output out of Git. Validate required pages, canonical
  production URLs, attribution markers, and internal links in CI.
- Add `CITATION.cff`, the official CC BY 4.0 text, a precise specification
  license boundary, and discoverability links from the workspace and crate
  READMEs.

## Rollout after merge

1. In GitHub repository settings, select **GitHub Actions** as the Pages source.
2. Add `specs.citymodel.3dgi.nl` as the Pages custom domain, then create DNS CNAME
   `specs.citymodel` pointing to `3dgi.github.io`.
3. Wait for certificate issuance and enable enforced HTTPS.
4. Verify the root, Arrow, Parquet, citation, and license pages, then create the
