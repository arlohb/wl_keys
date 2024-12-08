{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    eww.url = "github:elkowar/eww?ref=v0.6.0";
  };
  outputs = { self, nixpkgs, flake-utils, eww }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";

        name = "wl_keys";
        version = "0.1.0";
        deps = with pkgs; [
          wayland
          protobuf

          (eww.packages."${system}".default)
        ];

        package = pkgs.rustPlatform.buildRustPackage {
          inherit version;
          pname = name;

          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = deps;

          postInstall = ''
            cp -r ${./eww} $out/eww
          '';
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
