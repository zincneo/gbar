{
  description = "gbar - gpui status bar for niri";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        craneLib = crane.mkLib pkgs;

        # Filter source to include Rust files and local assets (png)
        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (craneLib.filterCargoSources path type) ||
            (builtins.match ".*\\.png$" path != null);
        };

        runtimeLibs = with pkgs; [
          openssl
          libxkbcommon
          libGL
          fontconfig
          freetype
          wayland
          libxcursor
          libxrandr
          libxi
          libx11
          libxcb
          glib
          gtk4
          gtk3
          vulkan-loader
        ];

        nativeBuildDeps = with pkgs; [
          pkg-config
        ];

        # Vendor cargo deps with a fix for gpui-component's proc macro
        # which needs ../assets/assets/icons relative to the crate dir.
        # Crane only extracts crate directories from git repos, so we
        # must manually include the workspace-level assets directory.
        cargoVendorDir = craneLib.vendorCargoDeps {
          inherit src;
          overrideVendorGitCheckout = ps: drv:
            if builtins.any (p: p.name == "gpui-component") ps then
              drv.overrideAttrs (old: {
                postInstall = (old.postInstall or "") + ''
                  if [ -d "crates/assets" ]; then
                    cp -rL "crates/assets" "$out/assets"
                  fi
                '';
              })
            else
              drv;
        };

        commonArgs = {
          inherit src cargoVendorDir;
          pname = "gbar";
          version = "0.1.0";

          nativeBuildInputs = nativeBuildDeps;
          buildInputs = runtimeLibs;

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibs;
        };

        # Build only the cargo dependencies for caching
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      in
      with pkgs; {
        packages.default = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;

          nativeBuildInputs = nativeBuildDeps ++ [ makeWrapper ];

          postInstall = ''
            wrapProgram $out/bin/gbar \
              --prefix LD_LIBRARY_PATH : "${lib.makeLibraryPath runtimeLibs}"
          '';
        });

        devShells.default = mkShell {
          buildInputs = runtimeLibs ++ [ pkg-config ];

          LD_LIBRARY_PATH = lib.makeLibraryPath runtimeLibs;
        };
      });
}
