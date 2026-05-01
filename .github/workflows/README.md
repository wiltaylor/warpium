# Release Workflow

This fork keeps a single GitHub Actions workflow: `create_release.yml`.

Push a tag matching `v*` to build release artifacts and attach them to the GitHub Release for that tag. The workflow infers the channel from the tag name:

* `*-dev` or `*-dev.*`: dev
* `*-preview` or `*-preview.*`: preview
* anything else matching `v*`: stable

The workflow can also be run manually with a `tag` input.

## Release Configuration

`release_configurations.json` defines the channel metadata used for GitHub Release names and prerelease status.

Fields still used by this fork:

* **channel**: The channel identifier.
* **is_prerelease**: Whether the GitHub Release is marked as a prerelease.
* **release_base_name**: The base name for GitHub Releases.
* **release_body_text**: The GitHub Release body text.
