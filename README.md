Declarative Testing Framework
===========================

[<img alt="github" src="https://img.shields.io/badge/github-austinsheep/demonstrate-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/austinsheep/demonstrate)
[<img alt="crates.io" src="https://img.shields.io/crates/v/demonstrate.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/demonstrate)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-demonstrate-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/demonstrate)
[<img alt="build status" src="https://img.shields.io/github/workflow/status/austinsheep/demonstrate/Continuous%20Integration/master?style=for-the-badge" height="20">](https://github.com/austinsheep/demonstrate/actions?query=branch%3Amaster)

Demonstrate allows tests to be written without as a much repetitive code within the `demonstrate!` macro, which will generate the corresponding full tests.

This testing library is highly influenced by [speculate.rs](https://github.com/utkarshkukreti/speculate.rs/) and [ruspec](https://github.com/k-nasa/ruspec/) which both take inspiration from [RSpec](https://rspec.info/).

The following new block definitions are utilized by Demonstrate:

- **`before`/`after`** — A block of source code that will be included at the start or end of each test respectively in the current and nested `describe`/`context` blocks.

- **`describe`/`context`** — `describe` and `context` are aliases for eachother. Specifies a new scope of tests which can contain a `before` and/or `after` block, nested `describe`/`context` blocks, and `it`/`test` blocks. These translate to Rust `mod` blocks, but also allow for shared test properties to be defined such as tests having outer attributes, being `async`, and having `Return<()>` types.

- **`it`/`test`** — `it` and `test` are aliases for eachother. Represents one test that translate to a Rust unit test.

<br />

## Example

```rust
use demonstrate::demonstrate;

fn is_4() -> u8 {
    4
}

demonstrate! {
    describe module {
        use super::is_4;

        before {
            let four = 4;
        }

        #[should_panic]
        it can_fail {
            assert!(four != 4)
        }

        test is_returnable -> Result<(), &'static str> {
            if is_4() == four {
                Ok(())
            } else {
                Err("It isn't 4! :o")
            }
        }

        #[async_attributes::test]
        async context asynchronous {
            before {
                let add_task = async_std::task::spawn(async {
                    1 + 1 + 1 + 1
                });
            }

            it awaits {
                assert_eq!(four, add_task.await)
            }
        }
    }
}
```

### License
<sup>
Licensed under <a href="LICENSE">MIT license</a>.
</sup>
