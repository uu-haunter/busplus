
# Use of Git/GitHub

When naming branches start off with a "tag" describing what you are working on (`feature`, `bugfix`, `hotfix`, `fix`, etc..) and follow with a description of what you are working on. It might also be good to specify that you are working on the `server`, `client` or some `general` work. Examples:

```
feature/server_actor_messaging
feature/general_guidelines_update
bugfix/server_no_connect_confirmation
hotfix/server_panic_client_disconnect
fix/client_typos_mapview
```

Feel free to choose whatever description suits your situation the best, but try to keep it short. In order to make the branch name more readable, make sure to use underscores (`_`) to separate words. Ex. `feature/this_is_a_good_branch_name` instead of `feature/thisisabadbranchname`.

When creating a pull request, make sure to link it to relevant issues since we are using `Projects` on GitHub where we have issues related to certain things that should be fixed/added.

A minimum of `(2)` people should have reviewed a pull request before merging it into the main branch. This is to be extra certain that the code meets our standards and that multiple people has had the chance to leave feedback on it.

When merging pull requests we `Squash and merge` to the main branch. This packs together all individual commits in the branch that is about to be merged into a single commit that will be added to the main branches commit history. And because we are squashing, we need to have well defined pull requests that fix/adds a specific part or feature of the code. In short, make sure that when creating a pull request, the branch you are merging only contains relevant work to feature you are adding/fixing.

When a pull request has been reviewed and merged, `remove` the branch from the upstream repository. We do this to keep the set of branches in the repository at a minimum, and those that are there only contain useful work.


# Review checklist

When reviewing a pull request, make sure that it meets the most important guidelines before merging:
- Does it pass CI?
- Is the feature/fix well defined and only solve a single or narrow group of problems?
- Are tests written where it is appropriate to have them?
- No magic constants, misleading variable names etc..

If you want a more precise list of things to look for/review, have a look at the following links:
- [Reviewer Checklist](https://devchecklists.com/pr-reviewer-checklist/)
- [PR Checklist](https://devchecklists.com/pull-requests-checklist/)

# CI

To make sure that the code we write meets a certain standard we will use Continuous Integration (CI).

The following things are qualities that should be checked on every pull request:
- That the code is correctly formatted (ex. `cargo fmt`).
- That the code compiles (ex. `cargo run`, `npm run build`).
- That any and all tests pass without fail (ex. `cargo test`, `npm run test`)

Things that might be worth adding in the future:
- Building the client and the server and making some requests between them with a test program (system test).

# Writing code in Rust

## Use statements

The rust formatter (`cargo fmt`) only provides a way to sort `use` statements alphabetically, but not in regards to their "origin" (`std`, `crate` or 3rd party crates).

Example: 
```rust
use crate::lobby::Lobby;
use crate::messages::{Connect, Disconnect, PositionUpdate, WsMessage};
use crate::protocol::client_protocol::ClientInput;
use actix::prelude::*;
use actix_web_actors::ws;
use std::time::{Duration, Instant};
use uuid::Uuid;
```

To keep things easy to read, sort use statements in the following order:
```rust
// 1) Standard library
use std::time::{Duration, Instant};

// 2) 3rd party crates
use actix::prelude::*;
use actix_web_actors::ws;
use uuid::Uuid;

// 3) Internal modules (preferred to be prefixed with "crate" but can be omitted).
use crate::lobby::Lobby;
use crate::messages::{Connect, Disconnect, PositionUpdate, WsMessage};
use crate::protocol::client_protocol::ClientInput;
```

## Comments/Documentation
Rust provides two ways to comment your code 
* Using two slashes, `//`, which is interpreted similar to white space (as in most programming languages).
* Using three slashes, `///`, which is a "doc" comment. Comments that start with three slashes are shown in the generated documentation

Document comments should be used when describing structs, methods, constants, etc. Normal comments are best suited to describe internal behaviour inside a function for example. See the following example:

```rust
/// A UTF-8–encoded, growable string.
pub struct String {
    vec: Vec<u8>,
}

impl String {
    /// Returns the length of this String, in bytes, not chars or graphemes.
    /// In other words, it may not be what a human considers the length of the string.
    pub fn len(&self) -> usize {
        // Returns the length of the internal vector.
        self.vec.len()
    }
}
```

When creating a new file, the first row(s) should contain a description of what is in that file using the `//!` comment, for example:

```rust
1 | //! Utilities for managing client connections.
2 |
3 | pub struct ClientConnection {
4 |     ...
5 | }
6 |  
7 | ...
```

See a detailed specification of comments/documentation in Rust [here](https://doc.rust-lang.org/reference/comments.html).