@ECHO off
echo Generating data . . .
md const_data >NUL 2>&1
cargo test generate --features gen-const-data

if %ERRORLEVEL% == 0 goto :compile
echo Error encountered during data generation.
goto :end

:compile
echo Compiling Hypersolve . . .
cargo build --features "" --release

:end
pause