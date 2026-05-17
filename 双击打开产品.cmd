@echo off
chcp 65001 >nul
setlocal

title Commerce Agents - Start Product
cd /d "%~dp0"

echo.
echo ========================================
echo   Commerce Agents - Start Product
echo ========================================
echo.
echo Project path: %cd%
echo.

where cargo >nul 2>nul
if errorlevel 1 goto cargo_missing

echo [1/4] Preparing backend service...
echo Checking Docker...
where docker >nul 2>nul
if errorlevel 1 goto docker_missing

docker ps >nul 2>nul
if not errorlevel 1 goto docker_ready

echo Docker Desktop is not ready. Trying to start it...
if exist "C:\Program Files\Docker\Docker\Docker Desktop.exe" (
  start "" "C:\Program Files\Docker\Docker\Docker Desktop.exe"
)
echo Waiting for Docker Desktop. First startup may take 1-3 minutes.
for /l %%i in (1,1,60) do (
  docker ps >nul 2>nul
  if not errorlevel 1 goto docker_ready
  timeout /t 5 /nobreak >nul
)
goto docker_timeout

:docker_ready
echo Setting up database and migrations...
cargo manage backend setup-db
if errorlevel 1 goto backend_setup_failed

echo Restarting backend service for this project...
cargo manage backend restart
if errorlevel 1 goto backend_start_failed

:backend_ready_after_start
echo.
echo [2/4] Restarting frontend web service for current source...
cargo manage frontend web restart
if errorlevel 1 goto frontend_start_failed

:frontend_ready_after_start
echo.
echo [3/4] Waiting for frontend HTTP response...
where curl.exe >nul 2>nul
if errorlevel 1 goto open_product

for /l %%i in (1,1,30) do (
  curl.exe -I -s "http://127.0.0.1:5173/" >nul 2>nul
  if not errorlevel 1 goto open_product
  timeout /t 2 /nobreak >nul
)
echo Warning: frontend HTTP probe did not respond yet. Browser will still be opened.

:open_product
echo.
echo [4/4] Opening Microsoft Edge Stable...
set "PRODUCT_URL=http://127.0.0.1:5173/commerce"
set "EDGE_STABLE_X86=C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"
set "EDGE_STABLE_X64=C:\Program Files\Microsoft\Edge\Application\msedge.exe"

if exist "%EDGE_STABLE_X86%" (
  start "" "%EDGE_STABLE_X86%" --new-window "%PRODUCT_URL%"
  goto success
)

if exist "%EDGE_STABLE_X64%" (
  start "" "%EDGE_STABLE_X64%" --new-window "%PRODUCT_URL%"
  goto success
)

echo Warning: Microsoft Edge Stable was not found. Trying Edge protocol...
start "" "microsoft-edge:%PRODUCT_URL%"
goto success

:success
echo.
echo Product opened:
echo   %PRODUCT_URL%
echo.
echo Login account and password:
echo   test123456
echo   test123456
echo.
echo Keep this window open if you want to watch startup logs.
echo To stop services later, run:
echo   cargo manage frontend web stop
echo   cargo manage backend stop
echo.
if not "%COMMERCE_AGENTS_NO_PAUSE%"=="1" pause
exit /b 0

:cargo_missing
echo [Error] cargo was not found. Install Rust / Cargo first.
goto fail

:docker_missing
echo [Error] docker was not found. Install Docker Desktop first.
goto fail

:docker_timeout
echo [Error] Docker Desktop did not become ready in time. Start Docker Desktop manually and run this file again.
goto fail

:backend_setup_failed
echo [Error] Database setup or migration failed.
goto fail

:backend_start_failed
echo [Error] Backend service failed to start.
goto fail

:frontend_start_failed
echo [Error] Frontend web service failed to start.
goto fail

:fail
echo.
if not "%COMMERCE_AGENTS_NO_PAUSE%"=="1" pause
exit /b 1
