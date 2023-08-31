# bwenv

## Installation

Using `cargo`
```sh
cargo install --git ssh://git@github.com/titanom/bwenv-rs.git
```

> **Note**  
> `cargo` uses libgit2 by default, which comes with many shortcomings regarding authentication.
> Make sure to set `net.git-fetch-with-cli = true` in your cargo config (`~/.cargo/config.toml`)

```toml
[net]
git-fetch-with-cli = true
```

## Usage

## Configuration

```toml
# list of environment variables to specify the environment being used
# first match applies
environment = ["MY_ENV", "NODE_ENV"]

[cache]
# directory to store the local cache
directory = "./node_modules/.cache"

# mage age in seconds of the local cache
max_age = 3600

# max seconds of stale cached values
stale_while_revalidate = 3600

[environments]
# specify named environments
[environments.development]
project = "bws_project_id"
alias = ["dev"]
cache_dir = "./node_modules/.dev-cache"
```