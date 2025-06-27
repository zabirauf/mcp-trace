# This is a template for submitting to nixpkgs
# Copy this to nixpkgs/pkgs/by-name/mc/mcp-trace/package.nix
{ lib
, rustPlatform
, fetchFromGitHub
, pkg-config
, stdenv
, darwin
}:

rustPlatform.buildRustPackage rec {
  pname = "mcp-trace";
  version = "0.1.0";

  src = fetchFromGitHub {
    owner = "zabirauf";
    repo = "mcp-trace";
    rev = "v${version}";
    hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
  };

  cargoHash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

  nativeBuildInputs = [ pkg-config ];

  buildInputs = lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
    darwin.apple_sdk.frameworks.SystemConfiguration
  ];

  # Build only the mcp-trace binary
  cargoBuildFlags = [ "--package" "mcp-trace" ];

  meta = with lib; {
    description = "A TUI to probe the calls between MCP client and server";
    homepage = "https://github.com/zabirauf/mcp-trace";
    changelog = "https://github.com/zabirauf/mcp-trace/releases/tag/v${version}";
    license = licenses.mit;
    maintainers = with maintainers; [ zabirauf ];
    mainProgram = "mcp-trace";
    platforms = platforms.unix;
  };
}