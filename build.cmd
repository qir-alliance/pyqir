@REM Copyright (c) Microsoft Corporation.
@REM Licensed under the MIT License.

@echo off

if '%1'=='/?' goto help
if '%1'=='-help' goto help
if '%1'=='-h' goto help

pwsh -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "& '%~dp0\eng\build.ps1' %*"
exit /B %errorlevel%

:help
pwsh -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "& '%~dp0\eng\build.ps1' -help"
