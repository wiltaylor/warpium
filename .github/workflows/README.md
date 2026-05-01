# Release Workflow

This fork keeps a single GitHub Actions workflow: `create_release.yml`.

Push to `master` with a release trailer on the HEAD commit to build release binaries and attach them to a GitHub Release.

Supported trailers:

* `release: true` creates a new release tag.
* `pre-release: true` creates a new prerelease tag.
* `re-release: true` rebuilds the latest `v*` tag.
* `re-release: v...` rebuilds a specific existing tag.

The workflow can also be run manually with a `tag` input to rebuild an existing tag, or with no tag to create a new release.

The release workflow only builds Linux and Windows Warpium app binaries with Cargo's release profile. It does not build macOS artifacts, installers, Linux packages, CLI archives, or web bundles.
