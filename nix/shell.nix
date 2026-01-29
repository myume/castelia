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
  nodejs,
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
    nodejs
  ];

  env.RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
}
