# AskHistorians Archive
[![CI](https://github.com/raffomania/aharc/workflows/CI/badge.svg)](https://github.com/raffomania/aharc/actions?query=workflow%3ACI)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/raffomania/aharc/blob/main/LICENSE-MIT)

[https://archive.observer](https://ask-historians-archive.netlify.app)

This is an *unofficial* archive of the subreddit [/r/AskHistorians](https://old.reddit.com/r/AskHistorians/). It's
ad-free, works on
mobile, loads fast, and doesn't need JS.

## Features

- Full-text search
- Collapse comments
- No AutoMod comments
- Only shows posts with actual answers

Posts are sorted in reverse chronological order and filtered to show only those with accepted answers. Since it's hosted
on the Netlify
free tier, it only contains posts made after {} for now.

## Project Status

The project is pretty much done for now. I'm actively reviewing pull requests, and I might revisit it in the future if
enough people request changes :) Here are
some ideas for improvements:

- Show nested comments
- Other ways to sort posts, like upvotes, number of answers, chronological order
- Option to hide read posts
- Add posts from other subreddits like AskScience

## Your Own Subreddit Archive

Requires `just`, `rustup` and `npm` to bootstrap all tools and configuration.

```bash
cargo install just
just init # setup repo, install required tools
```

This project takes ZSTD-compressed JSON dumps as found in the pushshift archives. Files need to be pre-processed before the tool can use them:

```bash
bin/preprocess-dump.sh <subreddit_submissions>.zst input/submissions.json
```

To run:
```bash
just run
```

To test:
```bash
just test
```

Before committing work:
```bash
just pre-commit
```

To see all available commands:
```bash
just list
```

## Similar Projects

The [redarc viewer](https://github.com/yakabuff/redarc) allows browsing pushshift dumps, but has a different UI than what I had in mind. It's also not a static site.

## License

This project is licensed under either of:
* MIT license ([LICENSE-MIT] or http://opensource.org/licenses/MIT)

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.


[LICENSE-MIT]: ./LICENSE-MIT
