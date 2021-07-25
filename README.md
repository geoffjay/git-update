[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

# git-update

On every new computer I have to setup my shell to include a function to easily update a repository,
for this I use:

```shell
git fetch --no-tags --all -p &&
(gff || true) &&
git branch -vv | grep ': gone]' | awk '{print $1}' | xargs -n 1 git branch -D &&
if [ "`git rev-parse --abbrev-ref HEAD`" != "staging" ]; then
    git fetch origin staging:staging
fi
```

which is fine, but why not doing something easier. The goal of this project is to do the same thing
with the command `git update`.

## License

[MIT](./LICENSE)
