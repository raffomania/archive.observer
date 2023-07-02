# aharc
[![CI](https://github.com/raffomania/aharc/workflows/CI/badge.svg)](https://github.com/raffomania/aharc/actions?query=workflow%3ACI)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/raffomania/aharc/blob/main/LICENSE-MIT)

# Development

Requires `just`, `rustup` and `npm` to bootstrap all tools and configuration.
For deployment, you need the `netlify` CLI installed.

```bash
cargo install just
just init # setup repo, install required tools
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

## Similar projects

The [redarc viewer](https://github.com/yakabuff/redarc) allows browsing pushshift dumps, but has a different UI than what I had in mind. It's also not a static site.

## License

This project is licensed under either of:
* MIT license ([LICENSE-MIT] or http://opensource.org/licenses/MIT)

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.


[LICENSE-MIT]: ./LICENSE-MIT
