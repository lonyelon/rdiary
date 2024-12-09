{ lib
, fetchFromGitHub
, rustPlatform
}:

rustPlatform.buildRustPackage rec {
  pname = "rdiary";
  version = "0.1.0";

  src = fetchFromGitHub {
    owner = "lonyelon";
    repo = "rdiary";
    rev = "v0.1.0";
    hash = "sha256-N6wyZdo3CL57DzzRx3Rc4iYwlUyrTYPJOaBLVKzSwUw=";
  };

  cargoLock.lockFile = src + /Cargo.lock;

  meta = {
    description = "Write your journal directly on the terminal.";
    homepage = "https://github.com/lonyelon/rdiary";
    license = lib.licenses.gpl3Only;
    platforms = lib.platforms.linux;
  };
}
