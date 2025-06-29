{
  description = "CEF Rust bindings development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        nativeBuildInputs = with pkgs; [
          rustToolchain
          cargo
          cmake
          pkg-config
          python3
          gdb
        ];

        buildInputs = with pkgs; [
          # CEF/Chromium dependencies
          gtk3
          glib
          nspr
          nss
          xorg.libX11
          xorg.libXcomposite
          xorg.libXcursor
          xorg.libXdamage
          xorg.libXext
          xorg.libXfixes
          xorg.libXi
          xorg.libXrandr
          xorg.libXrender
          xorg.libXtst
          xorg.libxcb
          libxkbcommon
          libGL
          libdrm
          mesa
          alsa-lib
          at-spi2-atk
          at-spi2-core
          atk
          cairo
          cups
          dbus
          expat
          fontconfig
          freetype
          gdk-pixbuf
          pango
          vulkan-loader
          libgbm
        ] ++ lib.optionals stdenv.isLinux [
          # Linux-specific dependencies
          systemd
          udev
        ];

        # Set up CEF environment
        cefPath = "$HOME/.local/share/cef";

      in
      {
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;

          shellHook = ''
            export CEF_PATH="${cefPath}"
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${cefPath}"
            export DYLD_FALLBACK_LIBRARY_PATH="$DYLD_FALLBACK_LIBRARY_PATH:${cefPath}"

            # Create CEF directory if it doesn't exist
            mkdir -p "${cefPath}"

            echo "CEF Rust development environment loaded"
            echo "CEF_PATH: $CEF_PATH"
            echo ""
            echo "To install CEF binaries, run:"
            echo "  cargo run -p export-cef-dir -- --force ${cefPath}"
          '';

          # Ensure libraries can be found
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        };

        # Package for building the project
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "cef-rs";
          version = "137.0.0+137.0.8";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit nativeBuildInputs buildInputs;

          # Skip tests that might require CEF binaries
          doCheck = false;

          meta = with pkgs.lib; {
            description = "CEF Rust bindings";
            homepage = "https://github.com/tauri-apps/cef-rs";
            license = with licenses; [ asl20 mit ];
            maintainers = [ ];
          };
        };
      });
}
