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
}
