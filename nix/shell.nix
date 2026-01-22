{
  mkShell,
  rust-analyzer,
  rustfmt,
  clippy,
  cargo,
  rustc,
  rustPlatform,
  watchexec,
  ffmpeg,
  sqlx-cli,
}:
mkShell {
  packages = [
    cargo
    rustc
    rustfmt
    rust-analyzer
    clippy
    watchexec
    ffmpeg
    sqlx-cli
  ];

  env.RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
}
