{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, crane, ... }: {
    devShells.aarch64-darwin.default = let
      pkgs = nixpkgs.legacyPackages.aarch64-darwin;
    in pkgs.mkShell {
      buildInputs = with pkgs; [
        darwin.apple_sdk.frameworks.SystemConfiguration
        iconv
        oci-cli
      ];
    };

    packages.x86_64-linux.default = let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      craneLibs = crane.mkLib pkgs;
    in craneLibs.buildPackage {
      src = craneLibs.cleanCargoSource ./.;
      buildInputs = with pkgs; [
        openssl
        pkg-config
      ];
    };

    overlays.rouxinold-autohalt = final: prev: {
      rouxinold-autohalt = self.packages.x86_64-linux.default;
    };

    nixosModules.rouxinold-autohalt = ./nix/modules/rouxinold-autohalt.nix;
  };
}
