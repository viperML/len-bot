{pkgs, ...}: {
  devShells.default = with pkgs;
    mkShell.override {} {
      packages = [
        cargo
        rustc
        pkg-config
        openssl
        rust-analyzer-unwrapped
        rustfmt
      ];

      RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
    };

  packages.default = with pkgs;
    rustPlatform.buildRustPackage {
      name = "len-bot";
      nativeBuildInputs = [pkg-config];
      buildInputs = [openssl];
      cargoLock.lockFile = ./Cargo.lock;
      src = ./.;
    };
}
