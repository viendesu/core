{
  inputs = {
    eva.url = "git+https://git.desu.church/VienDesu/eva.git";
    flake-utils.follows = "eva/flake-utils";
  };

  outputs = { eva, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system: {
      devShells.default = eva.lib.mkDevShell { pkgs = eva.lib.mkPkgs system; };
    });
}
