# Homebrew formula

Source of truth for the formula. Publish by copying to
[`baoulo/homebrew-tap`](https://github.com/baoulo/homebrew-tap) as `Formula/ditch.rb`.

Checksums are filled from the GitHub Release `SHA256SUMS` by
`sastri release update` (or `scripts/update-packaging-checksums.sh` as a fallback).

```bash
sastri release update v0.1.0 --push-channels --commit-back pr
brew install baoulo/tap/ditch
```
