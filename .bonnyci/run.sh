#!/bin/bash -xe

function install_deps() {
    echo "Installing Rust..."
    curl https://static.rust-lang.org/rustup.sh | sudo sh -s -- --spec=nightly
    echo "Cloning Rustbook..."
    git clone https://github.com/steveklabnik/rustbook.git
    cd rustbook
    echo "Building Rustbook..."
    cargo build --release
    export $PATH=$PATH:$PWD/rustbook/target/release/rustbook
}

function build_book(){
    rustbook build text/ book/
}

case "$BONNYCI_TEST_PIPELINE" in
    "check")
        install_deps
        build_book
    "gate")
        install_deps
        build_book
esac

exit 0

