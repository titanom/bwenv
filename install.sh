#!/bin/bash

set -e

RELEASE="latest"
OS="$(uname -s)"

if [ -d "$HOME/.bwenv" ]; then
  INSTALL_DIR="$HOME/.bwenv"
elif [ -n "$XDG_DATA_HOME" ]; then
  INSTALL_DIR="$XDG_DATA_HOME/bwenv"
elif [ "$OS" = "Darwin" ]; then
  INSTALL_DIR="$HOME/Library/Application Support/bwenv"
else
  INSTALL_DIR="$HOME/.local/share/bwenv"
fi

FILENAME=bwenv-x86_64-unknown-linux-gnu

download_bwenv() {
      URL="https://github.com/titanom/bwenv/releases/latest/download/$FILENAME.zip"

    DOWNLOAD_DIR=$(mktemp -d)

    echo "Downloading $URL..."

    mkdir -p "$INSTALL_DIR" &>/dev/null

    if ! curl --progress-bar --fail -L "$URL" -o "$DOWNLOAD_DIR/$FILENAME.zip"; then
      echo "Download failed.  Check that the release/filename are correct."
      exit 1
    fi

    unzip -q "$DOWNLOAD_DIR/$FILENAME.zip" -d "$DOWNLOAD_DIR"

    if [ -f "$DOWNLOAD_DIR/bwenv" ]; then
      mv "$DOWNLOAD_DIR/bwenv" "$INSTALL_DIR/bwenv"
    else
      mv "$DOWNLOAD_DIR/$FILENAME/bwenv" "$INSTALL_DIR/bwenv"
    fi

    chmod u+x "$INSTALL_DIR/bwenv"
}

check_dependencies() {
  echo "Checking dependencies for the installation script..."

  echo -n "Checking availability of curl... "
  if hash curl 2>/dev/null; then
    echo "OK!"
  else
    echo "Missing!"
    SHOULD_EXIT="true"
  fi

  echo -n "Checking availability of unzip... "
  if hash unzip 2>/dev/null; then
    echo "OK!"
  else
    echo "Missing!"
    SHOULD_EXIT="true"
  fi

  if [ "$SHOULD_EXIT" = "true" ]; then
    echo "Not installing bwenv due to missing dependencies."
    exit 1
  fi
}

ensure_containing_dir_exists() {
  local CONTAINING_DIR
  CONTAINING_DIR="$(dirname "$1")"
  if [ ! -d "$CONTAINING_DIR" ]; then
    echo " >> Creating directory $CONTAINING_DIR"
    mkdir -p "$CONTAINING_DIR"
  fi
}

setup_shell() {
  CURRENT_SHELL="$(basename "$SHELL")"

  if [ "$CURRENT_SHELL" = "bash" ]; then
    CONF_FILE=$HOME/.bashrc
    ensure_containing_dir_exists "$CONF_FILE"
    echo "Installing for Bash. Appending the following to $CONF_FILE:"
    echo ""
    echo '  # bwenv'
    echo '  export PATH="'"$INSTALL_DIR"':$PATH"'

    echo '' >>$CONF_FILE
    echo '# bwenv' >>$CONF_FILE
    echo 'export PATH="'"$INSTALL_DIR"':$PATH"' >>$CONF_FILE

  else
    echo "Could not infer shell type. Please set up manually."
    exit 1
  fi

  echo ""
  echo "In order to apply the changes, open a new terminal or run the following command:"
  echo ""
  echo "  source $CONF_FILE"
}

check_dependencies
download_bwenv
if [ "$SKIP_SHELL" != "true" ]; then
  setup_shell
fi
