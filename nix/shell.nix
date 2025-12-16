{
  mkShell,
  rust-analyzer,
  rustfmt,
  clippy,
  cargo,
  rustc,
  rustPlatform,
  watchexec,
}:
mkShell {
  packages = [
    cargo
    rustc
    rustfmt
    rust-analyzer
    clippy
    watchexec
  ];

  env.RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
}
