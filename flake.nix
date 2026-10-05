{
  inputs.eva.url = "git+https://git.desu.church/VienDesu/eva.git";

  outputs = { eva, ... }:
    eva.inputs.flake-utils.lib.eachDefaultSystem (system: {
      devShells.default = eva.lib.mkDevShell { pkgs = eva.lib.mkPkgs system; };
    });
}
