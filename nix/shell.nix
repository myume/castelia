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
  ];

  env.RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
}
