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

        custom-cef = pkgs.cef-binary.overrideAttrs (finalAttrs: previousAttrs: {
          version = "138.0.15";
          gitRevision = "d0f1f64";
          chromiumVersion = "138.0.7204.50";
          srcHash = "sha256-9MeJCV0Q2dnOeQ+C5QWBxD6PVzZh9wnhICGI8ak3SAM=";
          nativeBuildInputs = previousAttrs.nativeBuildInputs ++ [ pkgs.rsync ];
          installPhase = ''
            runHook preInstall

            cd ..
            mkdir -p $out
            cp -r Release/* $out
            cp -r Resources/* $out
            rsync ./* $out --exclude Release --exclude Resources

            runHook postInstall
          '';
        });

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
          custom-cef
        ] ++ lib.optionals stdenv.isLinux [
          # Linux-specific dependencies
          systemd
          udev
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;

          shellHook = ''
            export XDG_DATA_DIRS="$XDG_DATA_DIRS:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}"
          '';

          # Ensure libraries can be found
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          CEF_PATH = custom-cef;
          CEF_PATH_NO_CHECK = true;
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
