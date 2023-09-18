# bwenv

CLI for injecting secrets from Bitwarden Secrets Manager into a process

## Installation

### Using `cargo`
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

### Manual Download
Download the latest release from GitHub for your operating system.  
Available targets include:
- aarch64-apple-darwin (MacOS ARM)
- x84_64-unknown-linux-gnu (GNU-based x86_64 linux distros)

Unzip the archive & add it to PATH:
```sh
# replace <target> with the name of the downloaded archive
unzip ~/Downloads/<target>.zip
mv ~/Downloads/<target>/bwenv ~/.local/bin
chmod +x ~/.local/bin/bwenv
```


## Usage

```txt
Usage: bwenv [OPTIONS] [-- <SLOP>...]

Arguments:
  [SLOP]...

Options:
  -t, --token <TOKEN>      access token for the service account [env: BWS_ACCESS_TOKEN=]
  -p, --profile <PROFILE>  profile for loading project configuration [env: BWENV_PROFILE=]
  -h, --help               Print help (see more with '--help')
  -V, --version            Print version
```

### `<SLOP>`

Program in which the environment variables are injected.  
This can be any command, you would normally run in your shell, just prefixed with `bwenv [OPTIONS] --`.

### `token`

Access token for the service account of your project.  
Can be configured using the env variable `BWS_ACCESS_TOKEN` or using the `--token` option.  
Evaluation has following order:
1. `--token` option
2. `BWS_ACCESS_TOKEN` env variable

### `profile`

Profile for loading project configuration.  
Can be configured using the env variable `BWENV_PROFILE`, one of the variables defined in the configuration file or using the `--profile` option.  
Evaluation has following order:
1. `--profile` option
2. `BWENV_PROFILE` env variable
3. variables of the `environment` configuration starting with the first

## Configuration

```toml
environment = ["MY_ENV", "NODE_ENV"]

# default project if no profile option is provided, useful for local development
project = "<project-id>"

[cache]
# path to the cache directory
path = "./node_modules/.cache"

# max age in seconds, after which the cache is revalidated
max_age = 3600

# projects for specific profiles
[profiles]
[development]
project = "<project-id>"

[production]
project = "<project-id>"
```
