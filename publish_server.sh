

spacetime publish -c --project-path server network-game -y
rm client\src\module_bindings\*
spacetime generate --lang rust --out-dir client/src/module_bindings --project-path server