## SPDX-FileCopyrightText: 2026 Famedly GmbH
##
## SPDX-License-Identifier: Apache-2.0
{
  inputs = {
    famedly-engineering-standards.url = "github:famedly/engineering-standards";

    nixpkgs.follows = "famedly-engineering-standards/nixpkgs";
    flake-parts.follows = "famedly-engineering-standards/flake-parts";
  };

  outputs =
    { flake-parts, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.famedly-engineering-standards.flakeModules.default
      ];

      systems = inputs.famedly-engineering-standards.lib.famedlySystems;

      perSystem =
        { inputs', ... }:
        {
          devShells.default = inputs'.famedly-engineering-standards.devShells.rust;
        };
    };
}
