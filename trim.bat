@echo off
rem combine_all.bat – merge every *.rs and *.toml in this tree
setlocal enabledelayedexpansion

rem Output files
set "OUT_RS=target/combined.rs.out"
set "OUT_TOML=target/combined.toml.out"

if exist "%OUT_RS%" del "%OUT_RS%"
if exist "%OUT_TOML%" del "%OUT_TOML%"

rem -------- merge .rs --------
for /f "delims=" %%F in ('
    dir /b /s /o:n *.rs ^| findstr /v /i "\\target\\"
') do (
    echo /* --- %%~F --- */>>"%OUT_RS%"
    type "%%F" >>"%OUT_RS%"
    echo.>>"%OUT_RS%"
)

rem ----- merge .toml -----
for /f "delims=" %%F in ('
    dir /b /s /o:n *.toml ^| findstr /v /i "\\target\\"
') do (
    rem TOML uses # for comments
    echo # --- %%~F --- >>"%OUT_TOML%"
    type "%%F" >>"%OUT_TOML%"
    echo.>>"%OUT_TOML%"
)

echo Merged .rs files into %OUT_RS%
echo Merged .toml files into %OUT_TOML%
endlocal