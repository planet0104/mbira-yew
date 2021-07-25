@REM :: build
@REM wasm-pack build --no-typescript --target web
@REM @echo off
@REM for /f "tokens=4" %%a in ('route print^|findstr 0.0.0.0.*0.0.0.0') do (
@REM  set IP=%%a
@REM )
@REM :: cargo install simple-http-server
@REM simple-http-server -i --ip %IP% --port 8414

trunk serve --release