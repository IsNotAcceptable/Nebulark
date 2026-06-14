{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  outputs = { self, nixpkgs }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
  in {
    devShells.${system}.default = pkgs.mkShell {
      packages = with pkgs; [
        rustup cargo-edit cargo-watch
        pkg-config openssl
        wireguard-tools
        cross
        llvm clang
      ];
      RUST_LOG = "debug";
    };
  };
}