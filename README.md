[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![Build](https://github.com/geoffjay/git-update/actions/workflows/rust.yml/badge.svg)
![Release](https://github.com/geoffjay/git-update/actions/workflows/release.yml/badge.svg)

# git-update

`git-update` does the equivalent of:

```shell
git fetch --no-tags --all -p &&
(git merge --ff-only || true) &&
git branch -vv | grep ': gone]' | awk '{print $1}' | xargs -n 1 git branch -D &&
if [ "`git rev-parse --abbrev-ref HEAD`" != "staging" ]; then
    git fetch origin staging:staging
fi
```

Currently, this is just something I use to clean up after myself.

## Develop

The simplest way to install during development is with `cargo install --path .` which will add the
binary to the `cargo` bin directory.

## License

[MIT](./LICENSE)
