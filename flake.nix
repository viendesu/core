{
  inputs = {
    eva.url = "git+ssh://git@github.com/nerodono/eva.git";
    flake-utils.follows = "eva/flake-utils";
  };

  outputs = { eva, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system: {
      devShells.default = eva.lib.mkDevShell { pkgs = eva.lib.mkPkgs system; };
    });
}
