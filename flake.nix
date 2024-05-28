{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";

        name = "wl_keys";
        version = "0.1.0";
        deps = with pkgs; [
          protobuf
        ];

        package = pkgs.rustPlatform.buildRustPackage {
          inherit version;
          pname = name;

          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = deps;
        };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rust-analyzer
            clippy
            rustfmt
          ] ++ deps;
        };

        packages = rec {
          "${name}" = package;
          default = package;
        };
      }
    );
}
