# bwenv

CLI for injecting secrets from Bitwarden Secrets Manager into a process

## Installation

### Using `cargo`
Install latest version:
```sh
cargo install --git ssh://git@github.com/titanom/bwenv-rs.git
```

or a specific version (e.g. v1.0.0):
```sh
cargo install --git ssh://git@github.com/titanom/bwenv-rs.git --rev v1.0.0
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
Evaluation has the following order:
1. `--token` option
2. `BWS_ACCESS_TOKEN` env variable

### `profile`

Profile for loading project configuration.  
Can be configured using the env variable `BWENV_PROFILE`, one of the variables defined in the configuration file or using the `--profile` option.  
Evaluation has the following order:
1. `--profile` option
2. `BWENV_PROFILE` env variable
3. variables of the `environment` configuration starting with the first

## Configuration

The configuration file `bwenv.toml`, located in the root of your project must be used to configure profiles & caching behavior.  
This file should be committed, don't worry about leaking project IDs - they are not secret.
The only secret, you must *never* commit, is `BWS_ACCESS_TOKEN`, which therefore can not be configured using the config file.

```toml
environment = ["MY_ENV", "NODE_ENV"]

# default project if no profile option is provided, useful for local development
project = "<project-id>"
[override]
FORCE_COLOR = "1"

[cache]
# path to the cache directory
path = "./node_modules/.cache"

# max age in seconds, after which the cache is revalidated
max_age = 3600

# projects for specific profiles
[profile.development]
project = "<project-id>"
[profile.development.override]
FORCE_COLOR = "1"

[profile.production]
project = "<project-id>"
```

## Troubleshooting

### Network Issues & Bitwarden Incident

If for whatever reason, the Bitwarden API is not available - set `cache.max_age` to a very large number like 31556926 (1 year) to make sure the cache is always read.  

If it is your first time running `bwenv`, your only option is to manually retrieve the secrets from the Bitwarden Website and create the cache-file yourself.  

The location of the file is `<cache-path-from-config-file>/bwenv/<profile>.toml`.  
If you use the default project without a profile, replace `<profile>` with `no_profile`.
```toml
# replace this with the current UNIX timestamp
last_revalidation = 1694986302222

[variables]
KEY = "<value>"
OTHER_KEY = "<other-value>"
```
