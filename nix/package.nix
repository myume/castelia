{
  rustPlatform,
  lib,
}:
rustPlatform.buildRustPackage {
  pname = "castelia";
  version = "0.1.0";

  src = ../.;
  cargoLock.lockFile = ../Cargo.lock;

  buildInputs = [];

  meta = {
    description = "Self-hosted video broadcasting software";
    license = lib.licenses.agpl3Only;
    maintainers = with lib.maintainers; [myume];
    platforms = lib.platforms.all;
  };
}
