
@echo off
REM Script to publish the horror-game project using spacetime

spacetime publish -c --project-path server horror-game-test -y
rm client\src\module_bindings\*
spacetime generate --lang rust --out-dir client/src/module_bindings --project-path server