Dynamic Testing Framework
===========================

[<img alt="github" src="https://img.shields.io/badge/github-austinsheep/demonstrate-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/austinsheep/demonstrate)
[<img alt="crates.io" src="https://img.shields.io/crates/v/demonstrate.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/demonstrate)
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
    it is_root {
        assert!(true)
    }

    describe module {
        before {
            let four = 4;
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

        #[should_panic]
        it can_fail {
            assert!(four != 4)
        }
    }
}
```

### License
<sup>
Licensed under <a href="LICENSE">MIT license</a>.
</sup>
